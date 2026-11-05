#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
"""阶段同步的候选表：这一阶段哪些事实变了，仓里还有哪些现状句在说旧的。回扫员照表逐行判，不自己挑关键词、不整组放行。

用法（按次序）：
    stale-candidates.py --changes --base 提交 [--target 提交] --out 变更清单.md
        列出这一阶段在 kb 各文件「## 历史版本」节里新加的每一条变更记录（编号 H1、H2……），以及每个被改过的现状载体
        里删改前后的差异片段——写事实表的原料
        不给 --target 时结束取工作区，工作区或暂存区相对 HEAD 有改动（含没进 git 的新文件）就拒跑（退 2）：几个会话共写一个仓，
        别的会话没提交的改动会被当成这一阶段的。出路：用 `scripts/kb/batch-target.py` 把这一批的改动做成提交对象，再给 `--target <哈希>`；
        工作区或暂存区里有改动的路径全部登记在 `.claude/batch-scope`（常设登记、带理由）里的才照跑，输出头一行与变更清单头一行写明「全部登记在 batch-scope 里」；
        `.claude/batch-scope` 自己的改动不算（它就是登记处）
    stale-candidates.py --check-facts 事实表.tsv --base 提交 [--target 提交] [--scope 清单]
        核事实表罩没罩全：每条 H 编号都要出现在某一行的「出处」列里；每行的检索词要是合法的正则，
        而且在基准那一版的现状句里至少命中一行（旧说法当时就在那里）；新立事项反过来，检索词在基准那一版要零命中；
        每行「出处」列里点名的路径（带「:行号」的连行号）：已跟踪、工作区与暂存区和 HEAD 相同的放行；已跟踪而工作区或暂存区与 HEAD 不同
        （别的会话还没提交的改动）、又没登记在 .claude/batch-scope（这一批的改动范围，常设登记）的，判红、点名是哪一行
        （带行号的，那几行在 HEAD 里存在、工作区与暂存区的同一行与 HEAD 相同才放行；暂存区里的新文件同样红）；
        未跟踪的新文件（这一批新写的判决与证据）放行，输出里另列一行「依据是未跟踪的新文件 N 份：<名单>」（判法在 lib_basis_paths.py）
    stale-candidates.py --facts 事实表.tsv --base 提交 [--target 提交] --out 候选.tsv
        按事实表的检索词在全部现状载体里搜，每个命中的位置（载体:行号）一行候选，几条事实命中同一个位置的并成一行（候选表的格式见下面「候选表」）；
        records/ 下只扫 scripts/kb/stale-candidates-record-carriers.tsv 里登记的那几份（见下面「记录登记表」）；
        中文实验页上的产物行与「### 影响的决策」一节、改名登记表与术语表的行、代码围栏外的 markdown 标题行不出候选（见下面「不出候选的三类行」）
    stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]
        核报告有没有判到候选表里（分到的组里）的每一行
    stale-candidates.py --benchmark     拿仓里一段真实历史与一份盲写的事实表当阳性对照
    --scope 清单（--changes、--check-facts、--facts 都认）：一行一个从仓根起的路径（空行与 # 开头的行不算），
        「这一阶段改过哪些文件」只在清单里算——几个会话共写一个仓时，别的会话的改动不算进这一批的变更记录与差异片段；
        候选照旧搜全部现状载体。清单里的路径必须在 --base 到结束之间真的改过，有一个不在就退 2（清单写旧了、写错了，范围会悄悄变窄）
    stale-candidates.py --selftest

事实表（制表符分隔，首行是表头）：
    编号	旧事实	新事实	检索词	出处
    F1	层 0 的负载只有新池新建文件	层 0 两条流：……	层 0|崩溃点重放|录制流	H3;H17;提交 cae5092
  检索词是 Python 正则，写这件事实涉及的概念名词（「层 0」「录制流」「多次挂载」），新旧两种说法都会提到的那种；
  不写状态词，不照抄新说法的原句（「按整条流切段」只搜得到改好了的句子）；宁宽勿窄。
  几个词要同时出现就用 && 连（「E142&&改动计数」）；一行事实的检索词在结束那一版命中不许超过 500 行（MAXIMUM_CANDIDATE_LINES_PER_FACT），超了就是太宽，用 && 收窄。
  出处写这条事实来自哪几条变更记录（H 编号，--changes 给的）或哪个提交。
  旧事实写「只改措辞」的行只用来罩住变更记录，不出候选。
  旧事实以「新立：」开头的行是这一阶段之前仓里一个字都没提过的东西（新立的欠账、新门禁阶段），也不出候选；
  实现落地、实验跑完、欠账还清不是新立，是事实变了：旧事实写仓里原来怎么说它没有 / 还欠，检索词写那件事的概念名词。

候选表（--facts 写出，制表符分隔，表头不变）：
    组	旧事实	新事实	载体	原文
  一个「载体:行号」一行。组是命中这一行的事实里编号最小的那条（按数字比，F9 小于 F10；--groups 与 --check-report 按这一列认）；
  旧事实、新事实两列把命中这一行的每条事实都写上，每条前面带〔F7〕这样的编号，几条用「 ‖ 」隔开，编号小的在前。
  --check-report 按（组, 载体）配：并成一行之后，一个位置只判一次。

记录登记表（scripts/kb/stale-candidates-record-carriers.tsv）：
  records/ 下只有登记在这里的几份是现状载体，别的记录是事件史，不扫。一行「从仓根起的路径<制表符>理由」，空行与 # 开头的行不算；
  理由写这一份记录为什么装着开着的欠账。缺登记表，或有一行理由为空，--facts、--check-facts、--changes 退 2，点名登记表的那一行和出路；
  --facts 另核登记的路径在结束那一版里或工作区里有，两处都没有（写错了、搬走了）退 2，点名那一行和出路；
  只在工作区里有、结束那一版还没有的不算写错（回放更早的阶段时，后来才登记的记录那时还没有）。

不出候选的三类行（--facts 与 --benchmark 出候选时排掉；--check-facts 判检索词命中、--changes 出差异片段照旧读整份）：
  前两类（产物行、影响的决策一节）只管登记位里每个 kb 目录下 experiments/ 里的中文页（不以 .en.md、.ja.md 结尾）；第三类管每一份现状载体。
  产物行：去掉首尾空白之后，行首是 .claude/product-line-prefixes（工作区那一份）里登记的某个前缀，前面可以带一个反引号，
    或 grep 式的「文件:行号:」「行号:」前缀。登记表一行一个前缀（行首第一个词），空行与 # 开头的行不算，# 后是注释；
    登记表不在或一个前缀都没有，就不排这一类（只会多出候选，不会漏）。
  影响的决策一节：「### 影响的决策」标题之后、下一个标题之前的行；代码围栏里以 # 起头的行不算标题，围栏里的行照样算在节里。
  译文页不排：它们这一节的标题写法不一，没按节量过。
  术语表与围栏外标题（每一份现状载体，译文页同样排）：
    改名登记表与术语表：载体文件名（不含目录）是 term-renames*.md 或 translation-glossary*.md（三语都算），整份的行都不出候选——
      它们登记的是旧名与旧译法本身，旧说法留在里面是应当的。
    围栏外标题：行首是 1 到 6 个 # 加空白的 markdown 标题行，而且这一行不在代码围栏里（围栏从整份文件的第 1 行起数，``` 或 ~~~ 开头的行开关一次）。
      围栏里以 # 起头的行是复跑块里的命令注释，写的是起法，起法会过时，照出候选；只看「行以 # 开头」、不认围栏的那一版不能用。

报告（--check-report 判的就是这个）：
    ## 逐行判定
    | 组 | 载体 | 判定 | 改后的句子或理由 |
  候选表里每一行都要有一行；判定以「要改」「要补」「事件句不改」「不相干」「要人看」之一开头；
  最后一格不能空：要改的写改后的句子，其余写理由（至少 8 个字）。几份报告合起来判。
  另判一样（照 .claude/agents/sweep.md「产出」一节第一条）：报告里任何表格的行，只要有一格是 `crates/…:行号` 形态的载体、另有一格以「要改」或「要补」开头，
  而载体不是不变量清单（crates/singlefs-checker/invariants/ 下的 .md，那些留在原判定表里按文档行走写回员），就必须出现在报告里单列的「交实现员」一节里：
  一节从标题含「交实现员」的那一行起，到下一个同级或更高级标题止；节里要有同一个 `文件:行` 的写法（行号后面不再接数字）。几份报告的节合起来认。
  没有这样的行时本脚本不要求有这一节（sweep.md 要求写「0 行」和数行的命令，那一半靠主 agent 读报告）。

管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上）：
  写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地 / 实现这类字，过得了闸，
  这件事实的旧说法一行都进不了候选——判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表。
  逐行判的理由引了原话、判定也是五种之一，照样可以是没读懂就判的；靠主 agent 在全部判定里抽查复判。
  写表人把作用域词用 .* 拼进同一个正则（「宽词.*作用域|作用域.*宽词」）收窄，闸认不出：短语形态的检索词本来就这样写，分不开；靠主 agent 读事实表。

退出码：0 通过；2 参数或 git 出错；3 报告漏判、判定格式不对，或 crates/ 下判要改要补的行没进「交实现员」一节；4 阳性对照漏了已知的腐烂；5 自证失败；6 事实表没罩全。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一格的名字，括号里是那条「✗ 自证失败：」的开头；红时退 5）：
  STALE_CANDIDATES_BREAK=wide-unchecked  check_facts() 不判命中超过抽查门槛、出处列没写「宽词已抽查」的行
      宽词没抽查（「✗ 自证失败：命中超过抽查门槛、出处列没写「宽词已抽查」的事实表没被判红」）
  STALE_CANDIDATES_BREAK=lookahead-conjunction-unchecked  check_facts() 不认先行断言写的收窄（只认 &&）
      先行断言写的收窄（「✗ 自证失败：先行断言写的收窄（第一个词不宽）的事实表没被判红」）
  STALE_CANDIDATES_BREAK=dirty-worktree-unchecked  main() 不核「--changes 不给 --target 时工作区干净」
      脏工作区不给 target 没被拒（「✗ 自证失败：--changes 不给 --target、工作区有改动应当退 2」）、
      拒绝句没写出路（「✗ 自证失败：--changes 不给 --target 的拒绝没写 batch-target.py 的出路」）
      脏路径全登记在 batch-scope 里没照跑并在输出头一行写明（「✗ 自证失败：脏路径全登记在 batch-scope 里应当照跑并在输出头一行写明」）、
      脏路径有一个没登记没被拒（「✗ 自证失败：脏路径有一个没登记在 batch-scope 里应当退 2」）
  STALE_CANDIDATES_BREAK=unknown-option-ignored  main() 对认不出的选项不拒（用 parse_known_args 收下、丢掉）
      --changes 给一个认不出的选项没退 2（「✗ 自证失败：--changes 给认不出的选项应当退 2 且不写变更清单」）
  STALE_CANDIDATES_BREAK=provenance-unchecked  check_facts() 不核出处列里的路径有没有别的会话没提交的改动
      出处已跟踪且有没进这一批的改动（「✗ 自证失败：出处已跟踪且有没进这一批的改动的事实表应当退 6」）、出处引的行号被改了（「✗ 自证失败：出处引的行号被改了的事实表应当退 6」）、
      出处是暂存区里的新文件（「✗ 自证失败：出处是暂存区里的新文件的事实表应当退 6」）、
      点名是哪一行哪个路径（「✗ 自证失败：出处已跟踪且有改动的事实表没点名是哪一行哪个路径」）
      连带红的格：「✗ 自证失败：出处已跟踪且只暂存区有改动的事实表应当退 6」、「✗ 自证失败：出处是未跟踪新文件的事实表没另列一行名单」
  STALE_CANDIDATES_BREAK=invariants-carrier-missing  kb_directories() 不读登记位 .claude/kb-directories、退回写死的 .claude/kb（载体搬家之前的模式表），crates/singlefs-checker/invariants/ 下的 .md 不算载体
      不变量清单里的现状句没进候选（「✗ 自证失败：不变量清单（crates/singlefs-checker/invariants/ 下）里的现状句没进候选表」）、
      登记位写成 ./ 开头（「✗ 自证失败：登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表」）
  STALE_CANDIDATES_BREAK=handover-unchecked  check_reports() 不判 crates/ 下要改要补的行进没进「交实现员」一节
      crates/ 下判要改的行不在「交实现员」里（「✗ 自证失败：crates/ 下判「要改」的行不在「交实现员」一节里的报告没被判红」）、
      报告里根本没有「交实现员」一节（「✗ 自证失败：crates/ 下有判「要改」的行而报告没有「交实现员」一节，没被判红」）、判「要补」的行（「✗ 自证失败：crates/ 下判「要补」的行不在「交实现员」一节里的报告没被判红」）、
      crates/ 下不是不变量清单的 .md（「✗ 自证失败：crates/ 下不是不变量清单的 .md，判「要改」不在「交实现员」里的报告没被判红」）、
      只在别的小节里写了同一个位置（「✗ 自证失败：「交实现员」一节之外的小节里写了同一个位置，报告没被判红」）、
      节里的行号只是前缀（「✗ 自证失败：「交实现员」一节里写的行号只是这一行号的前缀，报告没被判红」）、点名是哪几行（「✗ 自证失败：crates/ 下判「要改」的行没进「交实现员」，拒绝说明没点名位置」）
  STALE_CANDIDATES_BREAK=record-carriers-unread  record_carrier_patterns() 不读记录登记表、退回扫 records/ 下全部记录，check_record_carriers_exist() 随之不核
      没登记的记录里的现状句进了候选（「✗ 自证失败：records/ 下没登记的记录里的现状句进了候选表」）、
      登记了哪儿都没有的路径没退 2（「✗ 自证失败：登记表里登记了哪儿都没有的路径，应当退 2、不写候选表、点名那一行与出路」）、
      有一行没写理由没退 2（「✗ 自证失败：登记表里有一行没写理由，应当退 2、不写候选表、点名那一行与出路」）、
      缺登记表没退 2（「✗ 自证失败：缺登记表，应当退 2、写明缺什么与出路」）
  STALE_CANDIDATES_BREAK=record-carrier-missing-unchecked  check_record_carriers_exist() 不核登记的路径在不在
      登记了哪儿都没有的路径没退 2（「✗ 自证失败：登记表里登记了哪儿都没有的路径，应当退 2、不写候选表、点名那一行与出路」）
  STALE_CANDIDATES_BREAK=record-carrier-reason-unchecked  read_record_carriers() 不核理由为空
      有一行没写理由没退 2（「✗ 自证失败：登记表里有一行没写理由，应当退 2、不写候选表、点名那一行与出路」）
  STALE_CANDIDATES_BREAK=duplicate-locations-kept  build_candidates() 不按位置合并，每条命中它的事实各出一行
      同一行命中两件事实没合成一行（「✗ 自证失败：同一行命中两件事实，候选表里没有合成一行」）、
      候选表里有位置出了不止一行（「✗ 自证失败：候选表里有位置出了不止一行」）、
      F9 与 F10 命中同一行时组号没取 F9（「✗ 自证失败：F9 与 F10 命中同一行时组号没取 F9」）
  STALE_CANDIDATES_BREAK=product-lines-kept  experiment_page_lines_left_out() 不排中文实验页上的产物行
      产物行不进候选（「✗ 自证失败：中文实验页上行首是产物行前缀的行进了候选表」）
  STALE_CANDIDATES_BREAK=impacted-decisions-kept  experiment_page_lines_left_out() 不排中文实验页上影响的决策一节里的行
      影响的决策一节不进候选（「✗ 自证失败：中文实验页影响的决策一节里的行进了候选表」）
  STALE_CANDIDATES_BREAK=glossary-lines-kept  carrier_lines_left_out() 不排改名登记表与术语表的行
      改名登记表与术语表里的行不出候选（「✗ 自证失败：改名登记表与术语表（term-renames*.md、translation-glossary*.md，三语）里的行进了候选表」）
  STALE_CANDIDATES_BREAK=headings-kept  carrier_lines_left_out() 不排代码围栏外的 markdown 标题行
      围栏外标题不出候选（「✗ 自证失败：代码围栏外的 markdown 标题行进了候选表」）
  STALE_CANDIDATES_BREAK=fence-comments-left-out  carrier_lines_left_out() 判标题时不认代码围栏，围栏里以 # 起头的行也当标题排掉
      围栏里的 # 行照出候选（「✗ 自证失败：代码围栏里以 # 起头的行、标题与术语表之外的行没进候选表」）
  QUOTE_KB_BREAK=kb-directories-raw  归 quote-kb.py（设进环境，kb_directories() 载入的那一份照认）：登记位的读法退回改前，原串不规整，`./crates/…` 拼出的模式配不上 git 路径
      登记位写成 ./ 开头（「✗ 自证失败：登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表」）
"""
import argparse
import contextlib
import difflib
import importlib.util
import io
import hashlib
import os
import re
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
preflight_sys.path.insert(0, preflight_os.path.dirname(preflight_os.path.realpath(__file__)))
import lib_basis_paths  # noqa: E402

CARRIER_PATTERNS = [
    r'^CLAUDE\.md$',
    r'^README\.md$',
    r'^\.claude/agent-common\.md$',
    r'^\.claude/main-agent\.md$',
    r'^\.claude/agents/[^/]+\.md$',
    r'^\.claude/rules/[^/]+\.md$',
    r'^\.claude/skills/[^/]+/SKILL\.md$',
    r'^\.claude/handover/[^/]+/README\.md$',
]
# records/ 下只有登记过的几份是现状载体（装着开着的欠账）；其余记录是事件史，扫它们只出噪声（四批阶段同步里 2410 行 records 候选，
# 落在两份拆分提案之外的 2322 行一处要改、要补、要人看都没有）。登记表一行「从仓根起的路径<制表符>理由」，空行与 # 开头的行不算
RECORD_CARRIERS_REGISTRY = 'scripts/kb/stale-candidates-record-carriers.tsv'
UNREGISTERED_RECORD_PATTERN = r'^records/[^/]+\.md$'   # 只给弄坏开关 record-carriers-unread 用：不读登记表、退回扫全部 records/
MERGED_FACT_SEPARATOR = ' ‖ '
# 不出候选的三类行。前两类在中文实验页上（records/2026-10-23-subagent拆分提案.md「一百六十、路径门禁与候选精度那一轮之后还开着的」⑤ 那一题）：
# 产物行是整行抄的冻结证据，本来就不许改；影响的决策一节的回看格是带日期的那一次的事，实验变了之后回看过没有归门禁 doc-experiments 的 decision-links 格判。
# 第三类在每一份现状载体上：改名登记表与术语表的行（登记的就是旧名与旧译法）、代码围栏外的 markdown 标题行；围栏里以 # 起头的是命令注释，照出候选。
# 只排出候选那一份语料（candidate_corpus）：check_facts 判检索词在基准那一版命中，用的是整页
PRODUCT_LINE_PREFIXES_REGISTRY = '.claude/product-line-prefixes'
PRODUCT_LINE_LEAD = r'^`?(?:[^\s`]*:\d+:\s*|\d+:)?'   # 产物行前缀前面许带的：一个反引号，或 grep 式的「文件:行号:」「行号:」
EXPERIMENT_PAGES_SUBDIRECTORY = 'experiments'
TRANSLATED_PAGE_SUFFIXES = ('.en.md', '.ja.md')   # 译文页这一节的标题有几种写法，没按节量过，不排
IMPACTED_DECISIONS_HEADING = re.compile(r'^### 影响的决策\s*$')
CODE_FENCE_MARKERS = ('```', '~~~')
GLOSSARY_PAGE_NAME = re.compile(r'^(?:term-renames|translation-glossary)[^/]*\.md$')   # 改名登记表与术语表的文件名（含 .en.md、.ja.md）
# kb 内容住在哪几个目录不在这里写死：读登记位 .claude/kb-directories（读法只在 quote-kb.py 的 kb_candidate_roots 一份，登记是替换不是追加），
# 登记的每个目录下的 .md 都是现状载体（carrier_patterns）；不变量清单 2026-10-11 拆出 .claude/kb 搬到 crates/singlefs-checker/invariants，靠登记位跟着认
KB_DIRECTORIES_FALLBACK = '.claude/kb'   # 只给弄坏开关 invariants-carrier-missing 用：不读登记位、退回写死的这一个目录
QUOTE_KB_SCRIPT = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'quote-kb.py')
CHANGE_LOG_ONLY_PATTERNS = [
    r'^\.claude/kb/decisions-history',
    r'^\.claude/kb/experiments-history\.md$',
]
HISTORY_SECTION_TITLE = '## 历史版本'
HISTORY_ENTRY_HEADING = re.compile(r'^### .+$', re.M)
LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')
# 交实现员：目标在 crates/ 下、判要改或要补的行，要在报告单列的这一节里（.claude/agents/sweep.md「产出」一节第一条）；不变量清单的 .md 例外
HANDOVER_SECTION_TITLE = '交实现员'
HANDOVER_VERDICTS = ('要改', '要补')
HANDOVER_LOCATION = re.compile(r'^crates/[^\s:|]+:\d+(?:[-,]\d+)*$')
HANDOVER_EXEMPT_PATH = re.compile(r'^crates/singlefs-checker/invariants/.+\.md$')
MARKDOWN_HEADING = re.compile(r'^(#{1,6})\s+(.*)$')
LISTED_HANDOVER_LINES = 10
MINIMUM_REASON_LENGTH = 8
FACT_TABLE_COLUMNS = ['编号', '旧事实', '新事实', '检索词', '出处']
NEWLY_ADDED_OLD_FACT_PREFIX = '新立：'
NOT_NEWLY_ADDED_WORDS = re.compile(r'落地|实现|已有|跑完|还清|有了')
REWORDING_ONLY_OLD_FACT_PREFIX = '只改措辞'
MAXIMUM_CANDIDATE_LINES_PER_FACT = 500
# 命中超过这么多行的检索词要先抽查精度：「出处」列里写一句「宽词已抽查：抽 N 行、相干 M 行」（推的门槛，没量过；
# 那一次「冻结」两组各 163 行逐行判下来全是不相干）
WIDE_TERM_SPOT_CHECK_LINES = 100
WIDE_TERM_SPOT_CHECK_NOTE = re.compile(r'宽词已抽查[：:][^\t]*\d')
REWORDING_HEADING_WORDS = re.compile(r'措辞|指代|写法|改名|搬|路径|一个字(不|没)改|错字|拼写|简称|格式对齐')
MINIMUM_QUOTED_CHARACTERS = 4
TERM_CONJUNCTION = '&&'


def compile_search_term(term):
    """「A&&B」→ 每一段各编一个正则，一行里全都命中才算命中。"""
    return [re.compile(part) for part in term.split(TERM_CONJUNCTION)]


def search_term_matches(compiled_parts, line):
    return all(part.search(line) for part in compiled_parts)


# 断言体以 . 起头的先行断言（「这一行别处有 / 没有某词」的同现写法）算并了第二个词；(?!\d) 这类只管词边界的不算
LOOKAHEAD_CONJUNCTION_OPENERS = ('(?=.', '(?!.')


def skip_escape_or_character_class(pattern, index):
    """pattern[index] 是 \\ 转义或 [...] 字符类的开头就返回它们之后的下标，否则返回 None。"""
    if pattern[index] == '\\':
        return index + 2
    if pattern[index] != '[':
        return None
    cursor = index + 1
    if cursor < len(pattern) and pattern[cursor] == '^':
        cursor += 1
    if cursor < len(pattern) and pattern[cursor] == ']':
        cursor += 1
    while cursor < len(pattern) and pattern[cursor] != ']':
        cursor += 2 if pattern[cursor] == '\\' else 1
    return cursor + 1


def group_end(pattern, start):
    """pattern[start] 是 (，返回与它配对的 ) 的下标；配不上返回 None。"""
    depth = 0
    index = start
    while index < len(pattern):
        skipped = skip_escape_or_character_class(pattern, index)
        if skipped is not None:
            index = skipped
            continue
        if pattern[index] == '(':
            depth += 1
        elif pattern[index] == ')':
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def remove_lookahead_conjunctions(pattern):
    """去掉断言体以 . 起头的先行断言整组，返回（剩下的正则，去掉了几组）。"""
    kept, removed, index = [], 0, 0
    while index < len(pattern):
        skipped = skip_escape_or_character_class(pattern, index)
        if skipped is not None:
            kept.append(pattern[index:skipped])
            index = skipped
            continue
        if pattern.startswith(LOOKAHEAD_CONJUNCTION_OPENERS, index):
            end = group_end(pattern, index)
            if end is not None:
                removed += 1
                index = end + 1
                continue
        kept.append(pattern[index])
        index += 1
    return ''.join(kept), removed

# 阳性对照：2026-09-18 知识腐烂回扫里主 agent 逐条现查坐实的 25 处腐烂（research/prompts/knowledge-rot-2026-09-18-sync.md），
# 行号是 BENCHMARK_TARGET 那一版的。只存「载体:行号」的 sha256 前 16 位：回扫员读得到这个文件，明文会被照抄成答案。
# 事实表 BENCHMARK_FACTS 由一个没读过这 25 处的 agent 照这一段历史的提交正文与变更记录盲写（第四版，
# research/prompts/sweep-acceptance-2026-09-18-v2-facts.md）；2026-09-19 三方第一轮之后按新闸把 9 行多余的 && 机械地退回第一个词
# （research/prompts/sweep-rel-r1-main-verification.md），罩住 25 处。对照守两样：罩住的不少于这个数、候选行数与冻结时相同。
# 答案在仓里另有明文（那份同步记录），对照只防工具被改窄，不当盲测用。
BENCHMARK_BASE = 'b1c8cef~1'
BENCHMARK_TARGET = '00c9d4f'
BENCHMARK_FACTS = 'scripts/kb/stale-candidates-benchmark-facts.tsv'
BENCHMARK_MINIMUM_COVERED = 25
# 候选行数按「records/ 只扫登记表里的、一个载体:行号一行、中文实验页排掉产物行与影响的决策一节、每份载体排掉改名登记表与术语表的行和代码围栏外的标题行」钉：两个数相等（一个位置只出一行）；
# 原先按事实各出一行是 4190 行、2821 个不同的行，实验页排掉那两类之前是 2483 行，再排掉第三类之前是 2410 行（第三类排掉的 87 行全是围栏外标题，这一段历史里还没有术语表）
BENCHMARK_EXPECTED_CANDIDATE_ROWS = 2323
BENCHMARK_EXPECTED_UNIQUE_LINES = 2323
BENCHMARK_KNOWN_STALE_LOCATION_HASHES = [
    '88600dee84e43cb1', '603dc36527671cbc', 'a93c0915fab7efa8', '948054ed6df42b0b', '7bb23ca4ed5f8432',
    '6287ca8bd47723bf', '201247a903f206ae', '16403eb11c52d13c', '82e4f62b752e4383', '84b95c231a2692a3',
    '767dc5b0a2eed64f', '43cec00ac6c7e52f', '7ec30e7996379b13', 'ef8cdf810547c225', '799d9c319db094c0',
    '8eb3cae28cc99670', 'bbed8920622afcc2', '4af43929ce1d4f2b', '9919777fff2f4391', '7c5fdaebc4e022d3',
    '873c19307057473f', '8dfe08b83d4f054f', '86b893c315e5795e', 'a3ae13e499fa522c', 'b4e110d8f010be83',
]


def normalize(text):
    """去掉空白、强调与表格记号，比原话时不受排版影响。"""
    return re.sub(r'[\s*`>#|]+', '', text)


def location_hash(location):
    return hashlib.sha256(location.encode('utf-8')).hexdigest()[:16]


def matches_any(path, patterns):
    return any(re.search(pattern, path) for pattern in patterns)


def kb_directories(repository_root):
    """登记位 .claude/kb-directories 里登记的 kb 目录（相对仓根）：读法是 quote-kb.py 的 kb_candidate_roots（没登记或登记的都不在就是 .claude/kb）。
    那边已经规整过（去掉开头的 ./、结尾的 /，拒掉绝对路径与走出仓的行、打到 stderr），拼进模式的是与 git 列出的路径同形的相对写法。"""
    if os.environ.get('STALE_CANDIDATES_BREAK') == 'invariants-carrier-missing':
        return [KB_DIRECTORIES_FALLBACK]   # 弄坏开关：不读登记位，退回载体搬家之前写死的 .claude/kb，不变量清单不扫
    spec = importlib.util.spec_from_file_location('quote_kb', QUOTE_KB_SCRIPT)
    quote_kb = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(quote_kb)
    return quote_kb.kb_candidate_roots(repository_root)


class RecordCarrierRegistryError(ValueError):
    """记录登记表读不了、有一行不合格、或登记的路径哪儿都没有：main() 退 2，打印原因与出路。"""

    def __init__(self, message, advice):
        super().__init__(message)
        self.advice = advice


def read_record_carriers(repository_root):
    """[(登记表里的行号, 路径, 理由)]：读 RECORD_CARRIERS_REGISTRY（工作区那一份）；空行与 # 开头的行不算，理由是路径后第一个制表符之后的整段。"""
    registry_path = os.path.join(repository_root, RECORD_CARRIERS_REGISTRY)
    if not os.path.isfile(registry_path):
        raise RecordCarrierRegistryError(
            f'缺记录登记表 {RECORD_CARRIERS_REGISTRY}（records/ 下只扫登记的那几份）',
            '新建它，一行「从仓根起的路径<制表符>理由」，理由写这一份记录为什么装着开着的欠账；空行与 # 开头的行不算')
    carriers = []
    with open(registry_path, encoding='utf-8') as handle:
        for number, line in enumerate(handle, start=1):
            line = line.rstrip('\n')
            if not line.strip() or line.lstrip().startswith('#'):
                continue
            path, _, reason = line.partition('\t')
            path, reason = path.strip(), reason.strip()
            if not reason and os.environ.get('STALE_CANDIDATES_BREAK') != 'record-carrier-reason-unchecked':
                raise RecordCarrierRegistryError(
                    f'{RECORD_CARRIERS_REGISTRY} 第 {number} 行「{line.strip()}」没写理由',
                    f'在路径后面加一个制表符和理由（这一份记录为什么装着开着的欠账）；这一份不该扫就删掉第 {number} 行')
            carriers.append((number, path, reason))
    return carriers


def record_carrier_patterns(repository_root):
    if os.environ.get('STALE_CANDIDATES_BREAK') == 'record-carriers-unread':
        return [UNREGISTERED_RECORD_PATTERN]   # 弄坏开关：不读登记表，退回扫 records/ 下全部记录
    return [rf'^{re.escape(path)}$' for _, path, _ in read_record_carriers(repository_root)]


def check_record_carriers_exist(repository_root, target):
    """登记的每个路径要在结束那一版里，或在工作区里（回放更早的阶段时，后来才有的登记记录在那一版里还没有，不算写错）；两处都没有就是写错了或搬走了。"""
    if os.environ.get('STALE_CANDIDATES_BREAK') in ('record-carriers-unread', 'record-carrier-missing-unchecked'):
        return
    paths = set(all_paths(repository_root, target))
    for number, path, _ in read_record_carriers(repository_root):
        if path not in paths and not os.path.isfile(os.path.join(repository_root, path)):
            raise RecordCarrierRegistryError(
                f'{RECORD_CARRIERS_REGISTRY} 第 {number} 行登记的 {path} 在 {target or "工作区"} 里不在，工作区里也没有',
                f'路径写错了就改成从仓根起的现路径；文件搬走了就改成新位置；记录已归档（git log 里找得到）就删掉第 {number} 行')


def carrier_patterns(repository_root):
    """现状载体的模式：CARRIER_PATTERNS、登记表里的每份记录、登记位里每个 kb 目录下的 .md（子目录也算）。"""
    return (CARRIER_PATTERNS + record_carrier_patterns(repository_root)
            + [rf'^{re.escape(directory)}/.+\.md$' for directory in kb_directories(repository_root)])


def is_current_state_carrier(path, patterns):
    return matches_any(path, patterns) and not matches_any(path, CHANGE_LOG_ONLY_PATTERNS)


def run_git(arguments, repository_root):
    completed = subprocess.run(['git', '-C', repository_root, '-c', 'core.quotepath=false', *arguments],
                               capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(f'git {" ".join(arguments)} 失败：{completed.stderr.strip()}')
    return completed.stdout


def read_version(repository_root, revision, path):
    """revision 为 None 读工作区；文件不在返回空串。"""
    if revision is None:
        full_path = os.path.join(repository_root, path)
        if not os.path.isfile(full_path):
            return ''
        with open(full_path, encoding='utf-8', errors='replace') as handle:
            return handle.read()
    completed = subprocess.run(['git', '-C', repository_root, 'show', f'{revision}:{path}'],
                               capture_output=True, text=True)
    return completed.stdout if completed.returncode == 0 else ''


def all_paths(repository_root, revision):
    if revision is None:
        paths = (run_git(['ls-files'], repository_root).split('\n')
                 + run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n'))
    else:
        paths = run_git(['ls-tree', '-r', '--name-only', revision], repository_root).split('\n')
    return sorted({path for path in paths if path})


CHANGE_SCOPE = None  # --scope 读进来的路径集合；None 表示整棵树


def read_change_scope(scope_path):
    with open(scope_path, encoding='utf-8') as handle:
        paths = {line.strip() for line in handle if line.strip() and not line.strip().startswith('#')}
    if not paths:
        raise ValueError(f'--scope 清单 {scope_path} 一个路径都没有')
    return paths


def unchanged_scope_paths(repository_root, base, target, scope):
    changed = set(changed_paths(repository_root, base, target, apply_scope=False))
    return sorted(scope - changed)


def changed_paths(repository_root, base, target, apply_scope=True):
    paths = run_git(['diff', '--name-only', base] + ([target] if target else []), repository_root).split('\n')
    if target is None:
        paths += run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n')
    paths = {path for path in paths if path}
    if apply_scope and CHANGE_SCOPE is not None:
        paths &= CHANGE_SCOPE
    return sorted(paths)


def current_state_lines(text):
    """历史版本节之前的行，(行号, 原文)。"""
    lines = []
    for line_number, line in enumerate(text.split('\n'), start=1):
        if line.strip() == HISTORY_SECTION_TITLE:
            break
        lines.append((line_number, line))
    return lines


def history_headings(text, path):
    """变更记录的标题：普通 kb 文件取「## 历史版本」之后的三级标题；变更史文件整份都是变更记录。"""
    if matches_any(path, CHANGE_LOG_ONLY_PATTERNS):
        section = text
    else:
        start = text.find('\n' + HISTORY_SECTION_TITLE)
        if start < 0:
            return []
        section = text[start:]
    return HISTORY_ENTRY_HEADING.findall(section)


def renamed_from(repository_root, base, target):
    """{新路径: 旧路径}：搬了目录、改了名的文件，历史标题要拿旧路径那一版去比。"""
    output = run_git(['diff', '--name-status', '-M', base] + ([target] if target else []), repository_root)
    renames = {}
    for line in output.split('\n'):
        columns = line.split('\t')
        if len(columns) == 3 and columns[0].startswith('R'):
            renames[columns[2]] = columns[1]
    return renames


def new_change_entries(repository_root, base, target):
    """[(H 编号, 文件, 标题)]，按文件与出现次序编号，同一 base / target 下稳定。"""
    entries = []
    renames = renamed_from(repository_root, base, target)
    for path in changed_paths(repository_root, base, target):
        if not path.startswith('.claude/kb/') or not path.endswith('.md'):
            continue
        base_path = renames.get(path, path)
        before = set(history_headings(read_version(repository_root, base, base_path), base_path))
        for heading in history_headings(read_version(repository_root, target, path), path):
            if heading not in before:
                entries.append((path, heading))
    return [(f'H{index}', path, heading) for index, (path, heading) in enumerate(entries, start=1)]


def changed_segments(base_text, target_text):
    """删改前后的差异片段 [(旧片段, 新片段)]：每一行被删的现状句配上同一文件新加的最像的一行，按字符比出改掉的那几段。"""
    base_lines = [line for _, line in current_state_lines(base_text)]
    target_lines = [line for _, line in current_state_lines(target_text)]
    target_set, base_set = set(target_lines), set(base_lines)
    added = [line for line in target_lines if line not in base_set and line.strip()]
    segments = []
    for removed in (line for line in base_lines if line not in target_set and line.strip()):
        replacement = max(added, default='', key=lambda candidate: difflib.SequenceMatcher(
            None, removed, candidate, autojunk=False).quick_ratio())
        matcher = difflib.SequenceMatcher(None, removed, replacement, autojunk=False)
        for operation, old_start, old_end, new_start, new_end in matcher.get_opcodes():
            if operation in ('replace', 'delete') and len(removed[old_start:old_end].strip()) >= 2:
                segments.append((removed[max(0, old_start - 12):old_end + 12].strip(),
                                 replacement[max(0, new_start - 12):new_end + 12].strip()))
    return segments


def write_changes(repository_root, base, target, output):
    output.write(f'# 变更清单 {base}..{target or "工作区"}\n\n## 变更记录（事实表的「出处」列要罩住每一个 H 编号）\n\n')
    output.write('| 编号 | 文件 | 标题 |\n|---|---|---|\n')
    for entry_id, path, heading in new_change_entries(repository_root, base, target):
        output.write(f'| {entry_id} | {path} | {heading[4:].replace("|", "｜")} |\n')
    output.write('\n## 现状载体里改掉的片段\n\n')
    patterns = carrier_patterns(repository_root)
    for path in changed_paths(repository_root, base, target):
        if not is_current_state_carrier(path, patterns):
            continue
        segments = changed_segments(read_version(repository_root, base, path), read_version(repository_root, target, path))
        if not segments:
            continue
        output.write(f'### {path}\n\n')
        for old_segment, new_segment in segments:
            output.write(f'- 旧：{old_segment[:200]}\n  新：{new_segment[:200]}\n')
        output.write('\n')


def read_fact_table(path):
    with open(path, encoding='utf-8') as handle:
        rows = [line.rstrip('\n').split('\t') for line in handle if line.strip()]
    if not rows or rows[0][:len(FACT_TABLE_COLUMNS)] != FACT_TABLE_COLUMNS:
        raise ValueError(f'{path} 的表头要是：{" / ".join(FACT_TABLE_COLUMNS)}（制表符分隔）')
    facts = []
    for row in rows[1:]:
        if len(row) < len(FACT_TABLE_COLUMNS):
            raise ValueError(f'{path} 这一行少列：{row[0] if row else "（空）"}')
        facts.append(dict(zip(FACT_TABLE_COLUMNS, row)))
    return facts


def current_state_corpus(repository_root, target):
    patterns = carrier_patterns(repository_root)
    return {path: current_state_lines(read_version(repository_root, target, path))
            for path in all_paths(repository_root, target) if is_current_state_carrier(path, patterns)}


def product_line_pattern(repository_root):
    """产物行的正则（配去掉首尾空白之后的行）：PRODUCT_LINE_LEAD 接 PRODUCT_LINE_PREFIXES_REGISTRY（工作区那一份）里登记的某个前缀。
    登记表不在或一个前缀都没有返回 None：不排产物行。"""
    registry_path = os.path.join(repository_root, PRODUCT_LINE_PREFIXES_REGISTRY)
    if not os.path.isfile(registry_path):
        return None
    prefixes = []
    with open(registry_path, encoding='utf-8') as handle:
        for line in handle:
            words = line.split('#', 1)[0].split()
            if words:
                prefixes.append(words[0])
    if not prefixes:
        return None
    return re.compile(PRODUCT_LINE_LEAD + '(?:' + '|'.join(re.escape(prefix) for prefix in prefixes) + r')\b')


def experiment_page_patterns(repository_root):
    """登记位里每个 kb 目录下 experiments/ 里的页（不进子目录）；译文页由 is_chinese_experiment_page 另排。"""
    return [rf'^{re.escape(directory)}/{EXPERIMENT_PAGES_SUBDIRECTORY}/[^/]+\.md$' for directory in kb_directories(repository_root)]


def is_chinese_experiment_page(path, patterns):
    return matches_any(path, patterns) and not path.endswith(TRANSLATED_PAGE_SUFFIXES)


def experiment_page_lines_left_out(lines, product_line):
    """一份中文实验页的现状句 [(行号, 原文)] 里不出候选的行号：产物行（product_line 为 None 时这一类不排），
    与影响的决策一节（IMPACTED_DECISIONS_HEADING 那一行之后、下一个标题之前）的行。代码围栏里以 # 起头的行不算标题。"""
    breaking = os.environ.get('STALE_CANDIDATES_BREAK')
    left_out = set()
    inside_fence = inside_impacted_decisions = False
    for line_number, line in lines:
        if line.strip().startswith(CODE_FENCE_MARKERS):
            inside_fence = not inside_fence
        elif not inside_fence and MARKDOWN_HEADING.match(line):
            inside_impacted_decisions = bool(IMPACTED_DECISIONS_HEADING.match(line))
            continue
        if inside_impacted_decisions and breaking != 'impacted-decisions-kept':   # 弄坏开关：影响的决策一节照出候选
            left_out.add(line_number)
        if product_line is not None and product_line.match(line.strip()) and breaking != 'product-lines-kept':   # 弄坏开关：产物行照出候选
            left_out.add(line_number)
    return left_out


def carrier_lines_left_out(path, lines):
    """任何一份现状载体的现状句 [(行号, 原文)] 里不出候选的行号：改名登记表与术语表（文件名配 GLOSSARY_PAGE_NAME，三语都算）的全部行，
    与别的载体里代码围栏外的 markdown 标题行。围栏里以 # 起头的行不算标题（复跑块里的命令注释，行里讲的起法会过时，照出候选）。"""
    breaking = os.environ.get('STALE_CANDIDATES_BREAK')
    if GLOSSARY_PAGE_NAME.match(os.path.basename(path)):
        return set() if breaking == 'glossary-lines-kept' else {line_number for line_number, _ in lines}   # 弄坏开关：术语表的行照出候选
    left_out = set()
    inside_fence = False
    for line_number, line in lines:
        if line.strip().startswith(CODE_FENCE_MARKERS):
            inside_fence = not inside_fence
        elif (not inside_fence or breaking == 'fence-comments-left-out') and MARKDOWN_HEADING.match(line):   # 弄坏开关：围栏里的 # 行当标题排掉
            if breaking != 'headings-kept':   # 弄坏开关：围栏外的标题照出候选
                left_out.add(line_number)
    return left_out


def candidate_corpus(repository_root, target):
    """出候选的语料：current_state_corpus 的现状句，去掉 carrier_lines_left_out 给的那几行（每份载体）
    与 experiment_page_lines_left_out 给的那几行（中文实验页）。"""
    pages = experiment_page_patterns(repository_root)
    product_line = product_line_pattern(repository_root)
    corpus = {}
    for path, lines in current_state_corpus(repository_root, target).items():
        left_out = carrier_lines_left_out(path, lines)
        if is_chinese_experiment_page(path, pages):
            left_out |= experiment_page_lines_left_out(lines, product_line)
        corpus[path] = [(line_number, line) for line_number, line in lines if line_number not in left_out]
    return corpus


def fact_number_key(fact_id):
    """事实编号按数字比：F9 小于 F10，不按字符串比。"""
    return [int(part) if part.isdigit() else part for part in re.split(r'(\d+)', fact_id)]


def build_candidates(repository_root, target, facts):
    """[(组号, 旧事实, 新事实, 载体:行号, 原文)]：一个「载体:行号」一行。
    组号取命中这一行的事实里编号最小的那条；旧事实、新事实两列把命中它的每条事实都写上，每条前面带〔编号〕，几条用 ‖ 隔开。
    行的次序：先按组号那条事实在事实表里的次序，同组按载体路径与行号。语料是 candidate_corpus（三类行不进，见文件头「不出候选的三类行」）。"""
    check_record_carriers_exist(repository_root, target)
    corpus = candidate_corpus(repository_root, target)
    searched = [(fact, compile_search_term(fact['检索词'])) for fact in facts
                if not fact['旧事实'].startswith((REWORDING_ONLY_OLD_FACT_PREFIX, NEWLY_ADDED_OLD_FACT_PREFIX))]
    hits = []   # [(载体:行号, 原文, [命中它的事实])]，载体路径与行号的次序
    for path, lines in corpus.items():
        for line_number, line in lines:
            matched = [fact for fact, compiled_parts in searched if search_term_matches(compiled_parts, line)]
            if matched:
                hits.append((f'{path}:{line_number}', line.strip(), matched))
    position = {id(fact): index for index, (fact, _) in enumerate(searched)}
    if os.environ.get('STALE_CANDIDATES_BREAK') == 'duplicate-locations-kept':
        # 弄坏开关：不合并，每条命中它的事实各出一行
        unmerged = [(fact['编号'], fact['旧事实'], fact['新事实'], location, line, position[id(fact)])
                    for location, line, matched in hits for fact in matched]
        unmerged.sort(key=lambda row: row[5])
        return [row[:5] for row in unmerged]
    candidates = []
    for location, line, matched in hits:
        matched = sorted(matched, key=lambda fact: fact_number_key(fact['编号']))
        candidates.append((matched[0]['编号'],
                           MERGED_FACT_SEPARATOR.join(f'〔{fact["编号"]}〕{fact["旧事实"]}' for fact in matched),
                           MERGED_FACT_SEPARATOR.join(f'〔{fact["编号"]}〕{fact["新事实"]}' for fact in matched),
                           location, line, position[id(matched[0])]))
    candidates.sort(key=lambda row: row[5])   # 稳定排序：同组的保持载体路径与行号的次序
    return [row[:5] for row in candidates]


def write_candidates(candidates, output):
    output.write('组\t旧事实\t新事实\t载体\t原文\n')
    for group, old_fact, new_fact, location, line in candidates:
        output.write(f'{group}\t{old_fact}\t{new_fact}\t{location}\t{line.replace(chr(9), " ")[:500]}\n')


def worktree_changes(repository_root):
    """工作区与暂存区相对 HEAD 的改动路径（含没进 git 的新文件，与不给 --target 时 changed_paths() 的口径一致）。"""
    paths = run_git(['diff', '--name-only', 'HEAD'], repository_root).split('\n')
    paths += run_git(['diff', '--cached', '--name-only', 'HEAD'], repository_root).split('\n')
    paths += run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n')
    return sorted({path for path in paths if path})


def dirty_worktree_gate(repository_root):
    """--changes 不给 --target 时：工作区干净、或有改动的路径全登记在 .claude/batch-scope 里，返回 None / 放行说明（要写在输出头一行）；拒绝时打印出路、返回退出码 2。
    .claude/batch-scope 自己的改动不算（它就是登记处）。"""
    if os.environ.get('STALE_CANDIDATES_BREAK') == 'dirty-worktree-unchecked':
        return None
    dirty = worktree_changes(repository_root)
    if not dirty:
        return None
    registered = lib_basis_paths.batch_scope_paths(repository_root) | {lib_basis_paths.BATCH_SCOPE}
    unregistered = [path for path in dirty if path not in registered]
    if not unregistered:
        return (f'⚠ --changes 没给 --target：工作区或暂存区相对 HEAD 有 {len(dirty)} 个路径的改动，'
                f'全部登记在 {lib_basis_paths.BATCH_SCOPE} 里，这些改动都当成这一阶段的')
    print(f'  ✗ --changes 没给 --target，而工作区或暂存区相对 HEAD 有 {len(unregistered)} 个路径的改动没登记在 {lib_basis_paths.BATCH_SCOPE} 里，'
          '别的会话没提交的改动会被当成这一阶段的：' + '、'.join(unregistered[:10]) + ('……' if len(unregistered) > 10 else ''))
    print('     → 怎么办：用 scripts/kb/batch-target.py 把这一批的改动做成提交对象，再给 --target <哈希>（写法在它的文件头）；'
          f'确实整个工作区都是这一批的，把这些路径逐条登记进 {lib_basis_paths.BATCH_SCOPE}（常设登记、带理由）')
    return 2


def check_facts(repository_root, base, target, fact_path):
    try:
        facts = read_fact_table(fact_path)
    except ValueError as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：照本脚本文件头「事实表」那一段的格式写，五列用制表符分开')
        return 6
    entries = new_change_entries(repository_root, base, target)
    cited = set()
    for fact in facts:
        cited.update(re.findall(r'H\d+', fact['出处']))
    uncovered = [(entry_id, path, heading) for entry_id, path, heading in entries if entry_id not in cited]
    base_corpus = current_state_corpus(repository_root, base)
    target_corpus = current_state_corpus(repository_root, target)
    heading_by_id = {entry_id: heading for entry_id, _, heading in entries}
    broken, empty, too_broad, not_new, fake_rewording, needless_conjunction, gone, wide_unchecked = [], [], [], [], [], [], [], []
    only_in_worktree, untracked = [], []
    committed = lib_basis_paths.CommittedState(repository_root)
    for fact in facts:
        if os.environ.get('STALE_CANDIDATES_BREAK') != 'provenance-unchecked':
            fact_problems, fact_untracked = lib_basis_paths.basis_check(repository_root, fact['出处'], committed)
            only_in_worktree += [f'{fact["编号"]} 行：{problem}' for problem in fact_problems]
            untracked += [path for path in fact_untracked if path not in untracked]
        try:
            compiled_parts = compile_search_term(fact['检索词'])
        except re.error as error:
            broken.append(f'{fact["编号"]}（{error}）')
            continue
        if fact['旧事实'].startswith(REWORDING_ONLY_OLD_FACT_PREFIX):
            cited_headings = [heading_by_id[entry_id] for entry_id in re.findall(r'H\d+', fact['出处']) if entry_id in heading_by_id]
            if any(not REWORDING_HEADING_WORDS.search(heading) for heading in cited_headings):
                fake_rewording.append(fact['编号'])
            continue
        if fact['旧事实'].startswith(NEWLY_ADDED_OLD_FACT_PREFIX):
            base_hits = any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines)
            if base_hits or NOT_NEWLY_ADDED_WORDS.search(fact['新事实']):
                not_new.append(fact['编号'])
            continue
        target_hits = sum(1 for lines in target_corpus.values() for _, line in lines
                          if search_term_matches(compiled_parts, line))
        if target_hits > MAXIMUM_CANDIDATE_LINES_PER_FACT:
            too_broad.append(f'{fact["编号"]}（{target_hits} 行）')
        if target_hits == 0:
            gone.append(fact['编号'])
        if (WIDE_TERM_SPOT_CHECK_LINES < target_hits <= MAXIMUM_CANDIDATE_LINES_PER_FACT and not WIDE_TERM_SPOT_CHECK_NOTE.search(fact['出处'])
                and os.environ.get('STALE_CANDIDATES_BREAK') != 'wide-unchecked'):
            wide_unchecked.append(f'{fact["编号"]}（{target_hits} 行）')
        first_part_text = fact['检索词'].split(TERM_CONJUNCTION)[0]
        first_word_text, lookaheads_removed = remove_lookahead_conjunctions(first_part_text)
        if os.environ.get('STALE_CANDIDATES_BREAK') == 'lookahead-conjunction-unchecked':
            lookaheads_removed = 0
        if lookaheads_removed:
            try:
                first_word = re.compile(first_word_text)
            except re.error as error:
                broken.append(f'{fact["编号"]}（去掉先行断言之后不是合法的正则：{error}）')
                continue
            first_word_hits = sum(1 for lines in target_corpus.values() for _, line in lines if first_word.search(line))
            if first_word_hits <= MAXIMUM_CANDIDATE_LINES_PER_FACT:
                needless_conjunction.append(f'{fact["编号"]}（去掉先行断言之后只命中 {first_word_hits} 行）')
        elif len(compiled_parts) > 1:
            first_part_hits = sum(1 for lines in target_corpus.values() for _, line in lines if compiled_parts[0].search(line))
            if first_part_hits <= MAXIMUM_CANDIDATE_LINES_PER_FACT:
                needless_conjunction.append(f'{fact["编号"]}（第一个词只命中 {first_part_hits} 行）')
        if not any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines):
            empty.append(fact['编号'])
    untracked_note = lib_basis_paths.untracked_note(untracked)
    if untracked_note:
        print(f'  ℹ {untracked_note}')
    if uncovered or broken or empty or too_broad or not_new or fake_rewording or needless_conjunction or gone or wide_unchecked or only_in_worktree:
        for entry_id, path, heading in uncovered[:40]:
            print(f'  ✗ {entry_id} 没有任何一行事实罩着：{path} {heading[4:80]}')  # gate-lint:detail
        if broken:
            print(f'  ✗ 检索词不是合法的正则：{"、".join(broken)}')  # gate-lint:detail
        if empty:
            print(f'  ✗ 检索词在基准那一版的现状句里一处都没命中（多半照抄了新说法）：{"、".join(empty)}')  # gate-lint:detail
        if not_new:
            print(f'  ✗ 标成「新立：」却不是新东西（检索词在基准那一版有命中，或新事实写着落地 / 实现 / 已有 / 跑完 / 还清）：'  # gate-lint:detail
                  f'{"、".join(not_new)}')
        if fake_rewording:
            print(f'  ✗ 标成「只改措辞」的行引的变更记录，标题里看不出只改了措辞：{"、".join(fake_rewording)}')  # gate-lint:detail
        if needless_conjunction:
            print(f'  ✗ 用了 && 或先行断言 (?=. / (?!. 而第一个词自己不超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行，不许再收窄：{"、".join(needless_conjunction)}')  # gate-lint:detail
        if gone:
            print(f'  ✗ 检索词在结束那一版一处都没命中（照抄了被这一阶段改掉的旧句）：{"、".join(gone)}')  # gate-lint:detail
        if too_broad:
            print(f'  ✗ 检索词太宽，结束那一版命中超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行：{"、".join(too_broad)}')  # gate-lint:detail
        if wide_unchecked:
            print(f'  ✗ 检索词命中超过 {WIDE_TERM_SPOT_CHECK_LINES} 行，「出处」列里没写「宽词已抽查：抽 N 行、相干 M 行」：{"、".join(wide_unchecked)}')  # gate-lint:detail
        for item in only_in_worktree:
            print(f'  ✗ 出处是别的会话还没提交的改动，不能当现状依据：{item}')  # gate-lint:detail
        print(f'  ✗ 事实表没罩全：{len(uncovered)} 条变更记录没罩、{len(broken)} 行正则坏了、'  # gate-lint:summary
              f'{len(empty)} 行检索词在基准零命中、{len(gone)} 行在结束零命中、{len(too_broad)} 行检索词太宽、'
              f'{len(needless_conjunction)} 行多余的 &&、{len(not_new)} 行冒充新立、{len(fake_rewording)} 行冒充只改措辞、'
              f'{len(wide_unchecked)} 行宽词没抽查、{len(only_in_worktree)} 处出处是别的会话还没提交的改动')
        print('     → 怎么办：读 --changes 给的变更清单，每条 H 编号写进它说的那件事实的「出处」列（只改措辞的也写一行，旧新事实写「只改措辞」）；'
              '检索词写新旧说法都会提到的概念名词，基准与结束两版都要命中；太宽（超过上限）才用 && 并上第二个概念名词收窄；'
              '事实真变了的不许写成只改措辞或新立；命中超过 ' + str(WIDE_TERM_SPOT_CHECK_LINES) + ' 行的先抽 10 行自判相干不相干，'
              '抽完在「出处」列写「宽词已抽查：抽 10 行、相干 N 行」，相干少也照写、不收窄；'
              '一行相干都没有的先查检索词是不是写偏了（照抄了新句、写了状态词、只是别的标识符的一段），写偏了换成这件事的概念名词；'
              '出处是别的会话还没提交的改动的：' + lib_basis_paths.ADVICE)
        return 6
    print(f'  ✓ 事实表罩全了：{len(entries)} 条变更记录都有出处，{len(facts)} 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）')
    return 0


def parse_group_ranges(text):
    """「F1-F12,F15」→ 组号集合；None 表示全部。"""
    if not text:
        return None
    selected = set()
    for part in text.split(','):
        match = re.fullmatch(r'([A-Z])(\d+)(?:-[A-Z]?(\d+))?', part.strip())
        if not match:
            if not re.fullmatch(r'[A-Z]\w*', part.strip()):
                raise ValueError(f'组号区间写不认得：{part}')
            selected.add(part.strip())
            continue
        low, high = int(match.group(2)), int(match.group(3) or match.group(2))
        if high < low or re.fullmatch(r'[A-Z]\d+-([A-Z])\d+', part.strip()) and part.strip().split('-')[1][0] != match.group(1):
            raise ValueError(f'组号区间写不认得（倒写，或两端字母不同）：{part}')
        selected.update(f'{match.group(1)}{number}' for number in range(low, high + 1))
    return selected


def table_rows_under(title, text):
    """某个二级标题下表格的数据行（去掉表头与分隔行），每行拆成单元格。"""
    rows, inside = [], False
    for line in text.split('\n'):
        if line.startswith('## '):
            inside = line.strip() == title
            continue
        if inside and line.startswith('|') and not re.match(r'^\|\s*-', line):
            rows.append([cell.strip().strip('`') for cell in line.strip().strip('|').split('|')])
    return [row for row in rows if row and row[0] != '组']


def read_candidate_rows(table_path):
    """[((组号, 载体:行号), 原文)]。"""
    rows = []
    with open(table_path, encoding='utf-8') as handle:
        next(handle)
        for row in handle:
            columns = row.rstrip('\n').split('\t')
            rows.append(((columns[0], columns[3]), columns[4] if len(columns) > 4 else ''))
    return rows


def quotes_the_line(reason, line):
    """理由里有没有这一行的原话（去掉空白与强调记号之后，至少 MINIMUM_QUOTED_CHARACTERS 个连续字符）。"""
    normalized_line, normalized_reason = normalize(line), normalize(reason)
    if len(normalized_line) < MINIMUM_QUOTED_CHARACTERS:
        return True
    return any(normalized_line[start:start + MINIMUM_QUOTED_CHARACTERS] in normalized_reason
               for start in range(len(normalized_line) - MINIMUM_QUOTED_CHARACTERS + 1))


def handover_scan(report_path, text):
    """一份报告里的两样东西：(「交实现员」一节里的各行, [(行号, 载体, 判定)])。
    后者是一节之外、代码围栏之外的表格行里，有一格是 crates/…:行号 形态的载体、另有一格以「要改」或「要补」开头的，不变量清单的 .md 不算。
    一节从标题含「交实现员」的那一行起，到下一个同级或更高级标题止。"""
    section_lines, rows = [], []
    inside_fence, inside_section, section_level = False, False, 0
    for number, line in enumerate(text.split('\n'), start=1):
        if line.strip().startswith(('```', '~~~')):
            inside_fence = not inside_fence
            if inside_section:
                section_lines.append(line)
            continue
        heading = None if inside_fence else MARKDOWN_HEADING.match(line)
        if heading:
            level = len(heading.group(1))
            if inside_section and level <= section_level:
                inside_section = False
            if not inside_section and HANDOVER_SECTION_TITLE in heading.group(2):
                inside_section, section_level = True, level
                continue
        if inside_section:
            section_lines.append(line)
            continue
        if inside_fence or not line.startswith('|'):
            continue
        cells = [cell.strip().strip('`*').strip() for cell in line.strip().strip('|').split('|')]
        location = next((cell for cell in cells if HANDOVER_LOCATION.match(cell)), None)
        verdict = next((cell for cell in cells if cell.startswith(HANDOVER_VERDICTS)), None)
        if location and verdict and not HANDOVER_EXEMPT_PATH.match(location.rsplit(':', 1)[0]):
            rows.append((number, location, verdict[:2]))
    return section_lines, rows


def handover_missing(report_paths):
    """→ (crates/ 下要改要补的行总数, [(报告路径, 行号, 载体, 判定)] 其中不在任何一份报告的「交实现员」一节里的)。几份报告的节合起来认。"""
    section_text, rows = [], []
    for report_path in report_paths:
        with open(report_path, encoding='utf-8') as handle:
            section_lines, report_rows = handover_scan(report_path, handle.read())
        section_text += section_lines
        rows += [(report_path, number, location, verdict) for number, location, verdict in report_rows]
    section = '\n'.join(section_text)
    missing = [row for row in rows if not re.search(re.escape(row[2]) + r'(?!\d)', section)]
    return len(rows), missing


def check_reports(table_path, report_paths, selected_groups=None):
    candidate_rows = read_candidate_rows(table_path)
    if selected_groups is not None:
        unknown = sorted(selected_groups - {key[0] for key, _ in candidate_rows})
        if unknown:
            print(f'  ✗ --groups 里有候选表没有的组：{"、".join(unknown[:20])}')
            print('     → 怎么办：组号照候选表第一列写，区间两端同一个字母、小号在前')
            return 2
        candidate_rows = [row for row in candidate_rows if row[0][0] in selected_groups]
    keys = [key for key, _ in candidate_rows]
    text_by_key = dict(candidate_rows)
    verdicts = {}
    for report_path in report_paths:
        with open(report_path, encoding='utf-8') as handle:
            for cells in table_rows_under('## 逐行判定', handle.read()):
                if len(cells) >= 3:
                    verdicts[(cells[0], cells[1])] = (cells[2], cells[3] if len(cells) > 3 else '')
    missing = [key for key in keys if key not in verdicts]
    malformed = []
    for key in keys:
        if key not in verdicts:
            continue
        verdict, reason = verdicts[key]
        if (not verdict.startswith(LINE_VERDICTS) or len(reason) < MINIMUM_REASON_LENGTH
                or not quotes_the_line(reason, text_by_key.get(key, ''))):
            malformed.append(key)
    handover_total, handover_gone = (0, []) if os.environ.get('STALE_CANDIDATES_BREAK') == 'handover-unchecked' else handover_missing(report_paths)
    if missing or malformed or handover_gone:
        if missing:
            print(f'  ✗ {len(missing)} 行候选没有逐行判定，例：'  # gate-lint:detail
                  + '；'.join(f'{group} {location}' for group, location in missing[:10]))
        if malformed:
            print(f'  ✗ {len(malformed)} 行的判定不以五种之一开头、最后一格不到 {MINIMUM_REASON_LENGTH} 个字、'  # gate-lint:detail
                  f'或最后一格没引这一行至少 {MINIMUM_QUOTED_CHARACTERS} 个字的原话，例：'
                  + '；'.join(f'{group} {location}' for group, location in malformed[:10]))
        if handover_gone:
            print(f'  ✗ {len(handover_gone)} 行目标在 crates/ 下、判了「要改」或「要补」，不在「交实现员」一节里，例：'  # gate-lint:detail
                  + '；'.join(f'{os.path.basename(report)}:{number} {location}（{verdict}）' for report, number, location, verdict in handover_gone[:LISTED_HANDOVER_LINES]))
        print(f'  ✗ 报告没判全：共 {len(keys)} 行候选，{len(missing)} 行没判、{len(malformed)} 行判定格式不对、'  # gate-lint:summary
              f'{len(handover_gone)} 行 crates/ 下要改要补的没进「交实现员」')
        print('     → 怎么办：在「## 逐行判定」表里给每一行候选一行 | 组 | 载体 | 判定 | 改后的句子或理由 |，'
              '判定以 要改 / 要补 / 事件句不改 / 不相干 / 要人看 开头，最后一格写改后的句子或理由，理由里引这一行的原话；'
              'crates/ 下判要改要补的行（不变量清单的 .md 除外）另在报告里单列一节「交实现员」，一行一条 文件:行 | 原行整行抄 | 判定 | 改后的句子或理由，'
              '位置的写法与上面表里那一格逐字相同（写法见 .claude/agents/sweep.md「产出」一节第一条）')
        return 3
    print(f'  ✓ 报告判全了：{len(keys)} 行候选都有逐行判定；crates/ 下判要改要补的 {handover_total} 行都在「交实现员」一节里（不变量清单的 .md 除外）')
    return 0


def benchmark(repository_root):
    facts_path = os.path.join(repository_root, BENCHMARK_FACTS)
    if not os.path.isfile(facts_path):
        print(f'  ✗ 缺阳性对照的事实表 {BENCHMARK_FACTS}')
        print('     → 怎么办：它随仓走；被删了就从 git 历史里取回，不要照着答案重写')
        return 4
    facts = read_fact_table(facts_path)
    candidates = build_candidates(repository_root, BENCHMARK_TARGET, facts)
    covered = {location_hash(location) for _, _, _, location, _ in candidates}
    covered_known = sum(1 for known in BENCHMARK_KNOWN_STALE_LOCATION_HASHES if known in covered)
    unique_lines = len({location for _, _, _, location, _ in candidates})
    if (len(candidates), unique_lines) != (BENCHMARK_EXPECTED_CANDIDATE_ROWS, BENCHMARK_EXPECTED_UNIQUE_LINES):
        print(f'  ✗ 阳性对照的候选数变了：{len(candidates)} 行、{unique_lines} 个不同的行，冻结时是 '
              f'{BENCHMARK_EXPECTED_CANDIDATE_ROWS} 行、{BENCHMARK_EXPECTED_UNIQUE_LINES} 个不同的行')
        print('     → 怎么办：同一份事实表、同一段历史，候选只该随候选规则变；是有意改了规则就重数并改这两个常量，写明为什么变')
        return 4
    if covered_known < BENCHMARK_MINIMUM_COVERED:
        print(f'  ✗ 阳性对照只罩住 {covered_known} 处已知的腐烂，低于 {BENCHMARK_MINIMUM_COVERED}（{BENCHMARK_BASE}..{BENCHMARK_TARGET}，只存哈希）')
        print('     → 怎么办：候选规则或盲写的事实表被改窄了；看 build_candidates、candidate_corpus 与 current_state_corpus 的改动，'
              '或那份事实表的检索词是不是被收窄过')
        return 4
    print(f'  ✓ 阳性对照：已知的 {len(BENCHMARK_KNOWN_STALE_LOCATION_HASHES)} 处腐烂里 {covered_known} 处在候选里（下限 {BENCHMARK_MINIMUM_COVERED}）'
          f'（盲写事实表 {len(facts)} 行，候选 {len(candidates)} 行、{unique_lines} 个不同的行）')
    return 0


def experiment_page_selftest_failures():
    """中文实验页上不出候选的两格（产物行一格、影响的决策一节一格），各判「该排的排了」与「同页别的行、译文页、别的载体照出」两面。
    自己一个临时 git 仓，结束取工作区；登记的前缀是样本自己的 E9SAMPLE，仓里登记的 E7RESULT 在样本里没登记、照出：前缀是从登记表现读的。返回失败句。"""
    failures = []
    with tempfile.TemporaryDirectory() as directory:
        def write(relative_path, content):
            full_path = os.path.join(directory, relative_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, 'w', encoding='utf-8') as handle:
                handle.write(content)

        subprocess.run(['git', '-C', directory, 'init', '-q'], check=True, capture_output=True)
        write(RECORD_CARRIERS_REGISTRY, '# 样本：没有登记的记录\n')
        write(PRODUCT_LINE_PREFIXES_REGISTRY, '# 样本：登记的产物行前缀\nE9SAMPLE  # 样本装置打的结果行\n')
        product_page = '.claude/kb/experiments/07-样本实验.md'
        write(product_page, '## E7 样本实验 —— 已跑\n\n'
              '层 0 只有一次挂载（正文里的现状句）。\n'                                   # 第 3 行：照出
              'E9SAMPLE name=层 0 的挂载次数 value=1\n'                                  # 第 4 行：产物行
              '`E9SAMPLE name=层 0 的挂载次数 value=1`\n'                                # 第 5 行：带反引号的产物行
              'research/results/e7-sample.out:12:E9SAMPLE name=层 0 的挂载次数 value=1\n'  # 第 6 行：带「文件:行号:」的产物行
              '12:E9SAMPLE name=层 0 的挂载次数 value=1\n'                               # 第 7 行：带「行号:」的产物行
              '正文里引了一段 E9SAMPLE name=层 0 的挂载次数，前缀不在行首。\n'             # 第 8 行：照出
              'E7RESULT name=层 0 的挂载次数 value=1\n')                                 # 第 9 行：样本没登记这个前缀，照出
        product_translation = '.claude/kb/experiments/07-sample-experiment.en.md'
        write(product_translation, '## E7 Sample experiment —— run\n\nE9SAMPLE name=层 0 的挂载次数 value=1\n')
        product_elsewhere = '.claude/kb/pitfalls.md'
        write(product_elsewhere, 'E9SAMPLE name=层 0 的挂载次数 value=1\n')
        decisions_page = '.claude/kb/experiments/08-另一样本实验.md'
        write(decisions_page, '## E8 另一样本实验 —— 已跑\n\n'
              '层 0 只有一次挂载（结论里的现状句）。\n\n'                                   # 第 3 行：照出
              '### 影响的决策\n\n| 决策分项 | 关系 | 回看 |\n|---|---|---|\n'
              '| D3（样本） 已定项 1 | 支撑：层 0 只有一次挂载 | 2026-01-01 不受影响：层 0 照旧 |\n'   # 第 9 行：节里的表行
              '```text\n# 层 0 这一行在代码围栏里，不是标题\n```\n'                          # 第 11 行：节里围栏中以 # 起头的行
              '层 0 在围栏之后、下一个标题之前。\n\n'                                        # 第 13 行：节里
              '### 它答不了的\n\n层 0 只有一次挂载（下一节里的现状句）。\n')                 # 第 17 行：照出
        decisions_translation = '.claude/kb/experiments/08-another-sample-experiment.ja.md'
        write(decisions_translation, '## E8 別のサンプル実験\n\n### 影响的决策\n\n| D3（样本） 已定项 1 | 支撑：层 0 只有一次挂载 | 2026-01-01 不受影响 |\n')
        write('facts.tsv', '\t'.join(FACT_TABLE_COLUMNS) + '\nF1\t层 0 只有一次挂载\t层 0 两条流\t层 0\tH1\n')
        locations = {row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts.tsv')))}
    product_left_out = {f'{product_page}:{number}' for number in (4, 5, 6, 7)}
    product_kept = {f'{product_page}:3', f'{product_page}:8', f'{product_page}:9', f'{product_translation}:3', f'{product_elsewhere}:1'}
    if product_left_out & locations:
        failures.append(f'中文实验页上行首是产物行前缀的行进了候选表（可带反引号或 文件:行号: / 行号: 前缀）：{sorted(product_left_out & locations)}')
    if product_kept - locations:
        failures.append('中文实验页上不该排的行没进候选表（产物行之外的行、前缀不在行首的、没登记的前缀起头的、译文页与别的载体里的产物行）：'
                        f'{sorted(product_kept - locations)}')
    decisions_left_out = {f'{decisions_page}:{number}' for number in (9, 11, 13)}
    decisions_kept = {f'{decisions_page}:3', f'{decisions_page}:17', f'{decisions_translation}:5'}
    if decisions_left_out & locations:
        failures.append(f'中文实验页影响的决策一节里的行进了候选表（含节里代码围栏中以 # 起头的行与围栏之后的行）：{sorted(decisions_left_out & locations)}')
    if decisions_kept - locations:
        failures.append(f'中文实验页影响的决策一节之外的行、译文页里同名一节的行没进候选表：{sorted(decisions_kept - locations)}')
    return failures


def carrier_lines_selftest_failures():
    """每一份现状载体上不出候选的第三类（改名登记表与术语表的行、代码围栏外的标题行），各判「该排的排了」与「该照出的照出」两面：
    围栏里以 # 起头的行、标题之外的正文、文件名只是含 term-renames 的载体。自己一个临时 git 仓，结束取工作区。返回失败句。"""
    failures = []
    with tempfile.TemporaryDirectory() as directory:
        def write(relative_path, content):
            full_path = os.path.join(directory, relative_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, 'w', encoding='utf-8') as handle:
                handle.write(content)

        subprocess.run(['git', '-C', directory, 'init', '-q'], check=True, capture_output=True)
        write(RECORD_CARRIERS_REGISTRY, '# 样本：没有登记的记录\n')
        glossary_pages = {
            '.claude/kb/term-renames.md': '# 改名登记\n\n| 层 0 | 旧名，已改 |\n',
            '.claude/kb/term-renames.ja.md': '| 层 0 | 旧名 |\n',
            '.claude/kb/translation-glossary.md': '## 术语\n\n层 0 译作 layer zero。\n',
            '.claude/kb/translation-glossary.en.md': '层 0 → layer zero\n',
        }
        for page, content in glossary_pages.items():
            write(page, content)
        heading_page = '.claude/kb/pitfalls.md'
        write(heading_page, '## 层 0 的挂载次数\n\n'                                       # 第 1 行：围栏外标题
              '层 0 只有一次挂载（正文里的现状句）。\n\n'                                   # 第 3 行：照出
              '```bash\n# 层 0 这一行在代码围栏里，是命令注释\necho 层 0\n```\n\n'           # 第 6、7 行：围栏里，照出
              '### 层 0 围栏之后的标题\n\n'                                               # 第 10 行：围栏外标题
              '正文里写 # 层 0 不在行首。\n')                                              # 第 12 行：照出
        similar_name_page = '.claude/kb/old-term-renames.md'
        write(similar_name_page, '层 0 只在文件名含 term-renames 的载体里。\n')            # 第 1 行：文件名不是登记表，照出
        write('facts.tsv', '\t'.join(FACT_TABLE_COLUMNS) + '\nF1\t层 0 只有一次挂载\t层 0 两条流\t层 0\tH1\n')
        locations = {row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts.tsv')))}
    glossary_in = {f'{page}:{number}' for page, content in glossary_pages.items()
                   for number, line in enumerate(content.split('\n'), start=1) if '层 0' in line} & locations
    if glossary_in:
        failures.append(f'改名登记表与术语表（term-renames*.md、translation-glossary*.md，三语）里的行进了候选表：{sorted(glossary_in)}')
    headings_in = {f'{heading_page}:{number}' for number in (1, 10)} & locations
    if headings_in:
        failures.append(f'代码围栏外的 markdown 标题行进了候选表：{sorted(headings_in)}')
    kept = {f'{heading_page}:{number}' for number in (3, 6, 7, 12)} | {f'{similar_name_page}:1'}
    if kept - locations:
        failures.append(f'代码围栏里以 # 起头的行、标题与术语表之外的行没进候选表：{sorted(kept - locations)}')
    return failures


def selftest():
    global MAXIMUM_CANDIDATE_LINES_PER_FACT, WIDE_TERM_SPOT_CHECK_LINES  # 下面临时把这两个门槛调成 0、判完换回
    with tempfile.TemporaryDirectory() as directory:
        def git(*arguments):
            subprocess.run(['git', '-C', directory, *arguments], check=True, capture_output=True)

        def write(relative_path, content):
            full_path = os.path.join(directory, relative_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, 'w', encoding='utf-8') as handle:
                handle.write(content)

        def quietly(function, *arguments):
            standard_output = sys.stdout
            with open(os.devnull, 'w') as silent:
                sys.stdout = silent
                try:
                    return function(*arguments)
                finally:
                    sys.stdout = standard_output

        git('init', '-q')
        git('config', 'user.email', 'selftest@example.invalid')
        git('config', 'user.name', 'selftest')
        write('CLAUDE.md', '# 项目\n\n层 0 的负载还只有新池新建文件。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('.claude/kb/table-before-move.md', '# 字节表\n\n' + '字段一行。\n' * 20 + '\n## 历史版本\n\n### 2026-08-30：搬家之前就有的条目\n')
        write('crates/singlefs-checker/invariants/01-sample.md', '层 0 只有一次挂载（不变量清单里的现状句）。\n')   # 载体搬家：不变量清单在 crates/ 下
        write('.claude/kb-directories', '.claude/kb  # 样本：决策、实验仍在这里\ncrates/singlefs-checker/invariants  # 样本：不变量清单搬到这里，登记位里登记了才扫\n')
        record_registry_text = ('# 样本：records/ 下登记为现状载体的记录\nrecords/2026-01-01-log.md\t样本：登记的记录，装着开着的欠账\n'
                                'records/2026-01-02-draft.md\t样本：登记的记录，这一份还没进 git\n')
        write(RECORD_CARRIERS_REGISTRY, record_registry_text)
        git('add', '-A')
        git('commit', '-q', '-m', 'base')
        os.makedirs(os.path.join(directory, '.claude/kb/layout'), exist_ok=True)
        os.rename(os.path.join(directory, '.claude/kb/table-before-move.md'),
                  os.path.join(directory, '.claude/kb/layout/01-first-txn.md'))
        write('CLAUDE.md', '# 项目\n\n层 0 的负载是两条流。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-02：层 0 扩成两条流\n\n### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('records/2026-01-01-log.md', '层 0 只有一次挂载的时候写的记录。\n')
        write('records/2026-01-06-unlisted.md', '层 0 只有一次挂载（这一份记录没登记在登记表里）。\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'stage')
        write('records/2026-01-02-draft.md', '层 0 只有一次挂载（还没进 git 的草稿）。\n')
        failures = []
        entries = new_change_entries(directory, 'HEAD~1', 'HEAD')
        if [heading for _, _, heading in entries] != ['### 2026-09-02：层 0 扩成两条流']:
            failures.append(f'新增变更记录没认对（改名的文件要按旧路径比）：{entries}')
        segments = changed_segments(read_version(directory, 'HEAD~1', 'CLAUDE.md'), read_version(directory, 'HEAD', 'CLAUDE.md'))
        if not any('只有新池新建文件' in old for old, _ in segments):
            failures.append('差异片段里没有被改掉的「只有新池新建文件」')
        header = '\t'.join(FACT_TABLE_COLUMNS) + '\n'
        good_row = 'F1\t层 0 只有新池新建文件\t层 0 两条流\t层 0\tH1\n'
        red_tables = {
            'facts-uncovered.tsv': (header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t层 0\t提交 abc\n', '漏了 H1'),
            'facts-empty.tsv': (header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t不存在的词\tH1\n', '检索词零命中'),
            'facts-new-wording.tsv': (header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t两条流\tH1\n', '检索词照抄新说法（基准零命中）'),
            'facts-gone.tsv': (header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t负载还只有\tH1\n', '检索词照抄被改掉的旧句（结束零命中）'),
            'facts-fake-new.tsv': (header + good_row + 'F2\t新立：层 0\t层 0 两条流\t层 0\tH1\n', '冒充新立（基准有命中）'),
            'facts-fake-new-words.tsv': (header + good_row + 'F2\t新立：两条流\t层 0 两条流落地\t两条流\tH1\n', '冒充新立（新事实写着落地）'),
            'facts-fake-rewording.tsv': (header + good_row + 'F2\t只改措辞\t只改措辞\t层 0\tH1\n', '冒充只改措辞（变更标题看不出改措辞）'),
            'facts-needless-and.tsv': (header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t层 0&&一次挂载\tH1\n', '第一个词不宽却用了 &&'),
            'facts-needless-lookahead.tsv': (header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t^(?=.*一次挂载).*层 0\tH1\n',
                                             '先行断言写的收窄（第一个词不宽）'),
        }
        write('facts-good.tsv', header + good_row)
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 0:
            failures.append('写对了的事实表被判红')
        write('facts-boundary-lookahead.tsv', header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t层 0(?![0-9])\tH1\n')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-boundary-lookahead.tsv')) != 0:
            failures.append('词边界断言 (?![0-9]) 被当成收窄，写对了的事实表被判红')
        for name, (content, what) in red_tables.items():
            write(name, content)
            if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, name)) != 6:
                failures.append(f'{what}的事实表没被判红')
        saved_limit = MAXIMUM_CANDIDATE_LINES_PER_FACT
        MAXIMUM_CANDIDATE_LINES_PER_FACT = 0
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 6:
            failures.append('命中行数超上限的事实表没被判红')
        MAXIMUM_CANDIDATE_LINES_PER_FACT = saved_limit
        saved_wide = WIDE_TERM_SPOT_CHECK_LINES
        WIDE_TERM_SPOT_CHECK_LINES = 0
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 6:
            failures.append('命中超过抽查门槛、出处列没写「宽词已抽查」的事实表没被判红')
        write('facts-wide-checked.tsv', header + 'F1\t层 0 只有新池新建文件\t层 0 两条流\t层 0\tH1；宽词已抽查：抽 10 行、相干 9 行\n')
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-wide-checked.tsv')) != 0:
            failures.append('写了「宽词已抽查」的宽检索词被判红')
        WIDE_TERM_SPOT_CHECK_LINES = saved_wide
        # 出处里点名的路径：已跟踪而工作区或暂存区与 HEAD 不同、又没登记在 .claude/batch-scope 的判红、点名是哪一行；
        # 与 HEAD 相同的、行号那几行没被改的、登记在 batch-scope 里的、放行的、未跟踪的新文件（这一批新写的判决与证据）判绿
        write('records/2026-01-01-log.md', '层 0 只有一次挂载的时候写的记录。\n工作区里新加的第二行。\n工作区里新加的第三行。\n')
        write('records/2026-01-04-staged-new.md', '别人暂存的新文件。\n')
        with open(os.path.join(directory, '.claude/kb/checks-owed.md'), encoding='utf-8') as handle:
            owed_at_head = handle.read()
        write('.claude/kb/checks-owed.md', owed_at_head + '别人暂存的一行。\n')
        git('add', 'records/2026-01-04-staged-new.md', '.claude/kb/checks-owed.md')
        write('.claude/kb/checks-owed.md', owed_at_head)   # 工作区改回 HEAD 的样子，只暂存区与 HEAD 不同
        provenance_cases = [
            ('facts-provenance-committed.tsv', 'H1；records/2026-01-01-log.md:1', 0, 'HEAD 里有、引的行号没被改的'),
            ('facts-provenance-untracked.tsv', 'H1；records/2026-01-02-draft.md', 0, '出处是未跟踪的新文件的'),
            ('facts-provenance-tracked.tsv', 'H1；records/2026-01-01-log.md', 6, '出处已跟踪且有没进这一批的改动的'),
            ('facts-provenance-line.tsv', 'H1；records/2026-01-01-log.md:3', 6, '出处引的行号被改了的'),
            ('facts-provenance-staged-new.tsv', 'H1；records/2026-01-04-staged-new.md', 6, '出处是暂存区里的新文件的'),
            ('facts-provenance-staged-only.tsv', 'H1；.claude/kb/checks-owed.md', 6, '出处已跟踪且只暂存区有改动的'),
            ('facts-provenance-nowhere.tsv', 'H1；records/2026-01-03-archived-long-ago.md', 0, '哪儿都没有的路径（早已归档）'),
        ]
        for name, provenance, want, what in provenance_cases:
            write(name, header + f'F1\t层 0 只有新池新建文件\t层 0 两条流\t层 0\t{provenance}\n')
            if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, name)) != want:
                failures.append(f'{what}事实表应当退 {want}')
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            check_facts(directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-provenance-tracked.tsv'))
        if 'F1 行' not in captured.getvalue() or 'records/2026-01-01-log.md' not in captured.getvalue():
            failures.append(f'出处已跟踪且有改动的事实表没点名是哪一行哪个路径：{captured.getvalue()[-400:]}')
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            check_facts(directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-provenance-untracked.tsv'))
        if '依据是未跟踪的新文件 1 份：records/2026-01-02-draft.md' not in captured.getvalue():
            failures.append(f'出处是未跟踪新文件的事实表没另列一行名单：{captured.getvalue()[-400:]}')
        write('.claude/batch-scope', '# 样本\nrecords/2026-01-02-draft.md\t# 这一批的改动\nrecords/2026-01-01-log.md\t# 这一批的改动\n'
              'records/2026-01-04-staged-new.md\t# 这一批的改动\n.claude/kb/checks-owed.md\t# 这一批的改动\n')
        for name in ('facts-provenance-tracked.tsv', 'facts-provenance-line.tsv', 'facts-provenance-staged-new.tsv', 'facts-provenance-staged-only.tsv'):
            if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, name)) != 0:
                failures.append(f'登记在 .claude/batch-scope 里的路径被判红（{name}）')
        os.remove(os.path.join(directory, '.claude/batch-scope'))
        git('reset', '-q')   # 把样本里暂存的两样放回去，后面的格不受影响
        write('facts-build.tsv', header + good_row + 'F2\t只改措辞\t只改措辞\t层 0\tH1\n'
              'F3\t层 0 只有新池新建文件\t层 0 两条流\t一次挂载\tH1\n')
        candidates = build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-build.tsv')))
        table_path = os.path.join(directory, 'candidates.tsv')
        with open(table_path, 'w', encoding='utf-8') as handle:
            write_candidates(candidates, handle)
        written = [key for key, _ in read_candidate_rows(table_path)]
        if ('F1', '.claude/kb/checks-owed.md:1') not in written:
            failures.append('还说「层 0 只有一次挂载」的现状句没进候选')
        if ('F1', 'records/2026-01-01-log.md:1') not in written:
            failures.append('records/ 下登记的记录里的现状句没写进候选表')
        if any(location.startswith('records/2026-01-06-unlisted.md:') for _, location in written):
            failures.append('records/ 下没登记的记录里的现状句进了候选表')
        if ('F1', 'crates/singlefs-checker/invariants/01-sample.md:1') not in written:
            failures.append('不变量清单（crates/singlefs-checker/invariants/ 下）里的现状句没进候选表')
        # 一个位置一行：F1（层 0）与 F3（一次挂载）都命中 checks-owed.md 第 1 行，合成一行，组号取编号小的 F1，旧事实一列两条都写上、各带〔编号〕
        shared_rows = [row for row in candidates if row[3] == '.claude/kb/checks-owed.md:1']
        if len(shared_rows) != 1:
            failures.append(f'同一行命中两件事实，候选表里没有合成一行：{len(shared_rows)} 行')
        elif shared_rows[0][0] != 'F1':
            failures.append(f'同一行命中两件事实，组号不是编号最小的那条：{shared_rows[0][0]}')
        elif not all(f'〔{number}〕' in shared_rows[0][1] and f'〔{number}〕' in shared_rows[0][2] for number in ('F1', 'F3')):
            failures.append(f'同一行命中两件事实，旧事实、新事实两列没把命中它的每条事实都带〔编号〕写上：{shared_rows[0][1:3]}')
        if len({row[3] for row in candidates}) != len(candidates):
            failures.append('候选表里有位置出了不止一行')
        # 编号按数字比：表里 F10 在前、F9 在后，命中同一行时组号是 F9，旧事实列先写 F9
        write('facts-numbered.tsv', header + 'F10\t旧十\t新十\t层 0\tH1\nF9\t旧九\t新九\t一次挂载\tH1\n')
        numbered_rows = [row for row in build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-numbered.tsv')))
                         if row[3] == '.claude/kb/checks-owed.md:1']
        if [row[0] for row in numbered_rows] != ['F9'] or not numbered_rows[0][1].startswith('〔F9〕旧九' + MERGED_FACT_SEPARATOR + '〔F10〕旧十'):
            failures.append(f'F9 与 F10 命中同一行时组号没取 F9（编号要按数字比，不按字符串比）：{[row[:2] for row in numbered_rows]}')
        if any(group == 'F2' for group, _ in written):
            failures.append('只改措辞的行出了候选')
        if any(location.startswith('.claude/kb/checks-owed.md:') and location != '.claude/kb/checks-owed.md:1'
               for _, location in written):
            failures.append('历史版本节里的句子进了候选')
        # 登记位写成 ./<目录>、带结尾的 /：读法在 quote-kb.py 的 kb_candidate_roots 规整成相对仓根的写法，拼出的模式照样配得上 git 列出的路径
        with open(os.path.join(directory, '.claude/kb-directories'), encoding='utf-8') as handle:
            registry_as_committed = handle.read()
        write('.claude/kb-directories', './.claude/kb/  # 样本：./ 开头、/ 结尾\n./crates/singlefs-checker/invariants  # 样本：./ 开头\n')
        dotted_path = os.path.join(directory, 'candidates-dotted-registry.tsv')
        with open(dotted_path, 'w', encoding='utf-8') as handle, contextlib.redirect_stderr(io.StringIO()):
            write_candidates(build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-build.tsv'))), handle)
        write('.claude/kb-directories', registry_as_committed)
        dotted = [key for key, _ in read_candidate_rows(dotted_path)]
        if ('F1', 'crates/singlefs-checker/invariants/01-sample.md:1') not in dotted or ('F1', '.claude/kb/checks-owed.md:1') not in dotted:
            failures.append(f'登记位写成 ./crates/singlefs-checker/invariants（./ 开头）时不变量清单里的现状句没进候选表：{dotted}')
        command_output = os.path.join(directory, 'candidates-from-command.tsv')
        subprocess.run([sys.executable, os.path.abspath(__file__), '--facts', os.path.join(directory, 'facts-build.tsv'),
                        '--base', 'HEAD~1', '--target', 'HEAD', '--out', command_output],
                       cwd=directory, capture_output=True, text=True)
        if not os.path.isfile(command_output) or read_candidate_rows(command_output) != read_candidate_rows(table_path):
            failures.append('--facts 命令写出的候选表与 build_candidates 的结果不一致')
        # 记录登记表有问题就退 2：登记的路径在结束那一版和工作区里都没有、理由为空、缺登记表；
        # 登记的路径只在工作区里有（上面登记的 draft 就是，结束那一版 HEAD 里还没有）不算写错
        next_registry_line = record_registry_text.count('\n') + 1

        def run_facts_with_registry(registry_content, name):
            registry_file = os.path.join(directory, RECORD_CARRIERS_REGISTRY)
            if registry_content is None:
                os.remove(registry_file)
            else:
                write(RECORD_CARRIERS_REGISTRY, registry_content)
            try:
                return subprocess.run([sys.executable, os.path.abspath(__file__), '--facts', os.path.join(directory, 'facts-build.tsv'),
                                       '--base', 'HEAD~1', '--target', 'HEAD', '--out', os.path.join(directory, name)],
                                      cwd=directory, capture_output=True, text=True)
            finally:
                write(RECORD_CARRIERS_REGISTRY, record_registry_text)

        not_there = run_facts_with_registry(record_registry_text + 'records/2026-01-09-not-there.md\t样本：登记了一个哪儿都没有的路径\n', 'registry-not-there.tsv')
        if (not_there.returncode != 2 or os.path.exists(os.path.join(directory, 'registry-not-there.tsv'))
                or 'records/2026-01-09-not-there.md' not in not_there.stdout or f'第 {next_registry_line} 行' not in not_there.stdout
                or '→ 怎么办' not in not_there.stdout):
            failures.append(f'登记表里登记了哪儿都没有的路径，应当退 2、不写候选表、点名那一行与出路：退 {not_there.returncode}，{not_there.stdout[-300:]}')
        no_reason = run_facts_with_registry(record_registry_text + 'records/2026-01-01-log.md\t\n', 'registry-no-reason.tsv')
        if (no_reason.returncode != 2 or os.path.exists(os.path.join(directory, 'registry-no-reason.tsv'))
                or f'第 {next_registry_line} 行' not in no_reason.stdout or '→ 怎么办' not in no_reason.stdout):
            failures.append(f'登记表里有一行没写理由，应当退 2、不写候选表、点名那一行与出路：退 {no_reason.returncode}，{no_reason.stdout[-300:]}')
        no_registry = run_facts_with_registry(None, 'registry-absent.tsv')
        if no_registry.returncode != 2 or '缺记录登记表' not in no_registry.stdout or '→ 怎么办' not in no_registry.stdout:
            failures.append(f'缺登记表，应当退 2、写明缺什么与出路：退 {no_registry.returncode}，{no_registry.stdout[-300:]}')        # --changes 不给 --target：工作区或暂存区相对 HEAD 有改动（此刻有没进 git 的草稿与改过的记录）就拒，给 --target 或放行参数才跑
        def run_changes(work_directory, output_path, *extra):
            return subprocess.run([sys.executable, os.path.abspath(__file__), '--changes', '--base', 'HEAD~1', '--out', output_path, *extra],
                                  cwd=work_directory, capture_output=True, text=True)

        def first_line(path):
            with open(path, encoding='utf-8') as handle:
                return handle.readline().rstrip('\n')

        refused = run_changes(directory, os.path.join(directory, 'dirty-no-target.md'))
        if refused.returncode != 2 or os.path.exists(os.path.join(directory, 'dirty-no-target.md')):
            failures.append(f'--changes 不给 --target、工作区有改动应当退 2 且不写变更清单：退 {refused.returncode}')
        if 'batch-target.py' not in refused.stdout or '--target <哈希>' not in refused.stdout:
            failures.append(f'--changes 不给 --target 的拒绝没写 batch-target.py 的出路：{refused.stdout[-300:]}')
        with_target = run_changes(directory, os.path.join(directory, 'dirty-with-target.md'), '--target', 'HEAD')
        if with_target.returncode != 0 or '⚠' in with_target.stdout or not os.path.isfile(os.path.join(directory, 'dirty-with-target.md')):
            failures.append(f'--changes 给了 --target、工作区有改动应当照跑、不打放行那一行：退 {with_target.returncode}，{with_target.stdout[-200:]}')
        refused_unknown = run_changes(directory, os.path.join(directory, 'unknown-option.md'), '--no-such-option-ever-used')
        if refused_unknown.returncode != 2 or os.path.exists(os.path.join(directory, 'unknown-option.md')) or '--no-such-option-ever-used' not in refused_unknown.stderr:
            failures.append(f'--changes 给认不出的选项应当退 2 且不写变更清单：退 {refused_unknown.returncode}，{refused_unknown.stderr[-200:]}')
        dirty_paths = worktree_changes(directory)
        write('.claude/batch-scope', '# 样本\n' + ''.join(f'{path}\t# 样本：这一批的改动\n' for path in dirty_paths))
        scoped_run = run_changes(directory, os.path.join(directory, 'dirty-scoped.md'))
        scoped_lines = scoped_run.stdout.splitlines()
        if scoped_run.returncode != 0 or not scoped_lines or not scoped_lines[0].startswith('⚠ ') or '登记在 .claude/batch-scope 里' not in scoped_lines[0]:
            failures.append(f'脏路径全登记在 batch-scope 里应当照跑并在输出头一行写明：退 {scoped_run.returncode}，{scoped_run.stdout[:200]}')
        elif not os.path.isfile(os.path.join(directory, 'dirty-scoped.md')) or first_line(os.path.join(directory, 'dirty-scoped.md')) != scoped_lines[0]:
            failures.append('脏路径全登记在 batch-scope 里放行时，变更清单头一行没写明')
        write('.claude/batch-scope', '# 样本\n' + ''.join(f'{path}\t# 样本：这一批的改动\n' for path in dirty_paths[:-1]))
        partial_run = run_changes(directory, os.path.join(directory, 'dirty-partial.md'))
        if partial_run.returncode != 2 or os.path.exists(os.path.join(directory, 'dirty-partial.md')) or dirty_paths[-1] not in partial_run.stdout:
            failures.append(f'脏路径有一个没登记在 batch-scope 里应当退 2、不写变更清单、点名没登记的 {dirty_paths[-1]}：退 {partial_run.returncode}，{partial_run.stdout[-300:]}')
        os.remove(os.path.join(directory, '.claude/batch-scope'))
        with tempfile.TemporaryDirectory() as clone_parent:
            clone = os.path.join(clone_parent, 'clone')
            subprocess.run(['git', 'clone', '-q', directory, clone], check=True, capture_output=True)
            clean_run = run_changes(clone, os.path.join(clone_parent, 'clean.md'))
            if clean_run.returncode != 0 or '⚠' in clean_run.stdout:
                failures.append(f'工作区干净时 --changes 不给 --target 应当照跑、不打放行那一行：退 {clean_run.returncode}，{(clean_run.stdout + clean_run.stderr)[-200:]}')
        worktree_locations = [row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts-good.tsv')))]
        if 'records/2026-01-02-draft.md:1' not in worktree_locations:
            failures.append('结束在工作区时，没进 git 的新文件没进候选')
        global CHANGE_SCOPE
        CHANGE_SCOPE = {'CLAUDE.md'}
        if new_change_entries(directory, 'HEAD~1', 'HEAD') != []:
            failures.append('--scope 只列 CLAUDE.md 时，范围外 checks-owed.md 的变更记录还被算进来')
        if changed_paths(directory, 'HEAD~1', 'HEAD') != ['CLAUDE.md']:
            failures.append('--scope 只列 CLAUDE.md 时，改过的文件没有正好剩下 CLAUDE.md')
        CHANGE_SCOPE = None
        if unchanged_scope_paths(directory, 'HEAD~1', 'HEAD', {'CLAUDE.md', 'README-never-changed.md'}) != ['README-never-changed.md']:
            failures.append('--scope 里写了这一阶段没改过的路径，没被认出来')
        if parse_group_ranges('F1-F2') != {'F1', 'F2'}:
            failures.append('组号区间没把两端都算进去')
        for bad_range in ('F2-F1', 'F1-G2'):
            try:
                parse_group_ranges(bad_range)
                failures.append(f'组号区间 {bad_range} 没被拒绝')
            except ValueError:
                pass
        rows = read_candidate_rows(table_path)
        def report(verdict, reason_of):
            return ('## 逐行判定\n| 组 | 载体 | 判定 | 改后的句子或理由 |\n|---|---|---|---|\n'
                    + ''.join(f'| {group} | {location} | {verdict} | {reason_of(line)} |\n' for (group, location), line in rows))
        write('complete.md', report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成层 0 的负载是两条流'))
        write('missing.md', report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成两条流').rstrip('\n').rsplit('\n', 1)[0] + '\n')
        write('unknown-verdict.md', report('同意', lambda line: '原话「' + line.replace('|', ' ') + '」照旧'))
        write('short-reason.md', report('不相干', lambda line: normalize(line)[:7]))
        write('no-quote.md', report('不相干', lambda line: '这一行说的是别的事情，与这件事实无关，不用改'))
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')]) != 0:
            failures.append('判全了的报告被判漏')
        for name, what in (('missing.md', '少判一行'), ('unknown-verdict.md', '判定不是五种之一'),
                           ('short-reason.md', '理由只有 7 个字'), ('no-quote.md', '理由没引这一行的原话')):
            if quietly(check_reports, table_path, [os.path.join(directory, name)]) != 3:
                failures.append(f'{what}的报告没被判红')
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')], {'F9'}) != 2:
            failures.append('--groups 里写了候选表没有的组，没被拒绝')
        # 交实现员：crates/ 下判要改要补的行要在单列的一节里（不变量清单的 .md 例外，候选表里 crates/singlefs-checker/invariants/01-sample.md 的行 complete.md 判了要改，照样判绿）
        complete_text = report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成层 0 的负载是两条流')
        code_location = 'crates/singlefs-verification/src/lib.rs:12'
        table_head = '\n## 其它命中\n| 文件:行 | 原行 | 判定 | 理由 |\n|---|---|---|---|\n'

        def code_row(location, verdict):
            return f'| {location} | // 层 0 只有新池新建文件 | {verdict} | 改成两条流 |\n'
        handover_with = lambda location: f'\n## 交实现员\n| {location} | // 层 0 只有新池新建文件 | 要改 | 改成两条流 |\n'
        handover_cases = (
            ('handover-missing.md', complete_text + table_head + code_row(code_location, '要改') + '\n## 交实现员\n0 行\n', 3,
             'crates/ 下判「要改」的行不在「交实现员」一节里的报告没被判红'),
            ('handover-no-section.md', complete_text + table_head + code_row(code_location, '要改'), 3,
             'crates/ 下有判「要改」的行而报告没有「交实现员」一节，没被判红'),
            ('handover-supplement.md', complete_text + table_head + code_row(code_location, '要补') + '\n## 交实现员\n0 行\n', 3,
             'crates/ 下判「要补」的行不在「交实现员」一节里的报告没被判红'),
            ('handover-readme.md', complete_text + table_head + code_row('crates/singlefs-verification/README.md:3', '要改') + '\n## 交实现员\n0 行\n', 3,
             'crates/ 下不是不变量清单的 .md，判「要改」不在「交实现员」里的报告没被判红'),
            ('handover-other-section.md', complete_text + table_head + code_row(code_location, '要改') + '\n## 交实现员\n0 行\n\n## 没做什么\n' + code_location + '\n', 3,
             '「交实现员」一节之外的小节里写了同一个位置，报告没被判红'),
            ('handover-line-prefix.md', complete_text + table_head + code_row(code_location, '要改') + handover_with(code_location + '0'), 3,
             '「交实现员」一节里写的行号只是这一行号的前缀，报告没被判红'),
        )
        for name, content, wanted_code, what in handover_cases:
            write(name, content)
            if quietly(check_reports, table_path, [os.path.join(directory, name)]) != wanted_code:
                failures.append(what)
        write('handover-named.md', complete_text + table_head + code_row(code_location, '要改'))
        named = io.StringIO()
        with contextlib.redirect_stdout(named):
            check_reports(table_path, [os.path.join(directory, 'handover-named.md')])
        if code_location not in named.getvalue() or 'handover-named.md:' not in named.getvalue() or '→ 怎么办' not in named.getvalue():
            failures.append('crates/ 下判「要改」的行没进「交实现员」，拒绝说明没点名位置')
        handover_green = (
            ('handover-ok.md', complete_text + table_head + code_row(code_location, '要改') + handover_with(code_location),
             'crates/ 下判「要改」的行写在「交实现员」一节里的报告被判红'),
            ('handover-ok-supplement.md', complete_text + table_head + code_row(code_location, '要补') + handover_with(code_location),
             'crates/ 下判「要补」的行写在「交实现员」一节里的报告被判红'),
            ('handover-ok-subheading.md', complete_text + table_head + code_row(code_location, '要改') + '\n### 交实现员\n' + f'- `{code_location}`：改成两条流\n\n#### 逐条\n{code_location}\n',
             '「交实现员」小节下的子标题里写的位置被判红'),
            ('handover-ok-empty.md', complete_text + '\n## 交实现员\n0 行（grep -c 命中数得 0）\n', '没有 crates/ 下要改要补的行、「交实现员」写 0 行的报告被判红'),
            ('handover-ok-none.md', complete_text, '没有 crates/ 下的行、也没有「交实现员」一节的报告被判红（本脚本不要求空节）'),
            ('handover-ok-invariants.md', complete_text + table_head + code_row('crates/singlefs-checker/invariants/02-other.md:4', '要改') + '\n## 交实现员\n0 行\n',
             '不变量清单的 .md 判「要改」、不在「交实现员」里被判红（它们按文档行走写回员）'),
            ('handover-ok-other-verdict.md', complete_text + table_head + code_row(code_location, '不相干') + code_row(code_location, '事件句不改'),
             'crates/ 下判「不相干」「事件句不改」的行被判红'),
            ('handover-ok-fence.md', complete_text + '\n```\n' + code_row(code_location, '要改') + '```\n', '代码围栏里的示例行被当成判定行判红'),
        )
        for name, content, what in handover_green:
            write(name, content)
            if quietly(check_reports, table_path, [os.path.join(directory, name)]) != 0:
                failures.append(what)
        # 几份报告合起来认：判定行在一份、「交实现员」节在另一份
        write('handover-split-a.md', complete_text + table_head + code_row(code_location, '要改'))
        write('handover-split-b.md', handover_with(code_location))
        if quietly(check_reports, table_path, [os.path.join(directory, 'handover-split-a.md'), os.path.join(directory, 'handover-split-b.md')]) != 0:
            failures.append('判定行在一份报告、「交实现员」节在另一份报告时被判红（几份报告合起来判）')
    failures += experiment_page_selftest_failures()
    failures += carrier_lines_selftest_failures()
    if failures:
        for failure in failures:
            print(f'  ✗ 自证失败：{failure}')  # gate-lint:detail
        print('  ✗ stale-candidates 自证没过')  # gate-lint:summary
        print('     → 怎么办：对照 selftest、experiment_page_selftest_failures 与 carrier_lines_selftest_failures 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、'
              'build_candidates、candidate_corpus、experiment_page_lines_left_out、carrier_lines_left_out、check_reports 哪一支被改了')
        return 5
    print('  ✓ stale-candidates 自证通过：变更记录（含改名）与差异片段认对；事实表八种写法各判红、写对的判绿、超上限判红；'
          '候选收 records/ 下登记的记录（没登记的不进）、crates/singlefs-checker/invariants/ 下的不变量清单（登记位写成 ./ 开头、/ 结尾也认）与没进 git 的文件、'
          '同一行命中几件事实只出一行（组号取编号最小的、编号按数字比，旧新事实两列带〔编号〕写全）、只改措辞不出候选、历史版本节不进；'
          '中文实验页上行首是登记前缀的产物行（带反引号、文件:行号:、行号: 前缀的也算）与影响的决策一节里的行（节里围栏中的 # 行不算标题）不进，'
          '同页别的行、前缀不在行首的、没登记的前缀、译文页与别的载体照进；'
          '改名登记表与术语表（三语）的行、代码围栏外的标题行不进，围栏里以 # 起头的行、标题之外的正文、文件名只是含 term-renames 的载体照进；'
          '记录登记表登记了哪儿都没有的路径、有一行没写理由、缺登记表都退 2 并点名那一行与出路；'
          '组号区间含两端、倒写与跨字母拒绝、写了不存在的组拒绝；报告漏判、判定不认得、理由太短、理由没引原话都判红，--facts 命令写出的表与函数结果一致；'
          'crates/ 下判要改要补的行不在「交实现员」一节里（没有这一节、写在别的小节、行号只是前缀、不是不变量清单的 .md）判红并点名位置，写在节里（含子标题下）、不变量清单的 .md、'
          '判了不相干或事件句不改、代码围栏里的示例、没有这样的行判绿，判定行与节分在几份报告里合起来认；'
          '--changes 不给 --target 时工作区有改动就拒（出路写 batch-target.py）、给 --target 照跑、给认不出的选项退 2、脏路径全登记在 batch-scope 里照跑并在输出头一行写明、有一个没登记拒并点名、工作区干净照跑；'
          '--scope 把范围外的变更记录挡在外面、范围内的照算、写了没改过的路径认得出；出处里点名的路径已跟踪而工作区或暂存区有没进这一批的改动判红（点名哪一行；引的行号被改、暂存区里的新文件同样红）、与 HEAD 相同、行号没被改、登记在 batch-scope、未跟踪的新文件（另列一行名单）判绿')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--base')
    parser.add_argument('--target')
    parser.add_argument('--out')
    parser.add_argument('--changes', action='store_true')
    parser.add_argument('--check-facts', metavar='事实表')
    parser.add_argument('--facts', metavar='事实表')
    parser.add_argument('--check-report', nargs='+', metavar='文件')
    parser.add_argument('--groups')
    parser.add_argument('--scope', metavar='清单')
    parser.add_argument('--benchmark', action='store_true')
    parser.add_argument('--selftest', action='store_true')
    if os.environ.get('STALE_CANDIDATES_BREAK') == 'unknown-option-ignored':   # 弄坏开关：认不出的选项收下丢掉
        arguments = parser.parse_known_args()[0]
    else:
        arguments = parser.parse_args()
    if arguments.selftest:
        return selftest()
    try:
        repository_root = run_git(['rev-parse', '--show-toplevel'], os.getcwd()).strip()
        if arguments.benchmark:
            return benchmark(repository_root)
        if arguments.check_report:
            if len(arguments.check_report) < 2:
                print('  ✗ --check-report 要候选表与至少一份报告')
                print('     → 怎么办：stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]')
                return 2
            return check_reports(arguments.check_report[0], arguments.check_report[1:], parse_group_ranges(arguments.groups))
        if not arguments.base:
            print('  ✗ 缺 --base')
            print('     → 怎么办：给阶段开始之前的提交，例：--base b1c8cef~1；用法见本脚本文件头')
            return 2
        if arguments.scope:
            global CHANGE_SCOPE
            scope = read_change_scope(arguments.scope)
            stale = unchanged_scope_paths(repository_root, arguments.base, arguments.target, scope)
            if stale:
                print(f'  ✗ --scope 清单里有 {len(stale)} 个路径在 {arguments.base} 到结束之间没改过：' + '、'.join(stale[:20]))
                print('     → 怎么办：清单只列这一批真改过（含新建、删除）的文件；写错了路径或清单是旧的，改清单再跑')
                return 2
            CHANGE_SCOPE = scope
        if arguments.check_facts:
            return check_facts(repository_root, arguments.base, arguments.target, arguments.check_facts)
        dirty_notice = None
        if arguments.changes and not arguments.target:
            refusal = dirty_worktree_gate(repository_root)
            if isinstance(refusal, int):
                return refusal
            dirty_notice = refusal
        if not arguments.out:
            print('  ✗ 缺 --out')
            print('     → 怎么办：--changes 与 --facts 都要 --out 指一个新文件（排他新建）')
            return 2
        if not arguments.changes and not arguments.facts:
            print('  ✗ 缺 --changes 或 --facts')
            print('     → 怎么办：先 --changes 出变更清单、写事实表、--check-facts 过了，再 --facts 出候选表')
            return 2
        if dirty_notice:
            print(dirty_notice)
        # 先把内容算完再排他新建输出文件：算的过程中退 2（事实表、记录登记表有问题）不留一个空文件挡住改好之后的重跑
        if arguments.changes:
            content = io.StringIO()
            if dirty_notice:
                content.write(dirty_notice + '\n\n')
            write_changes(repository_root, arguments.base, arguments.target, content)
            entry_total = len(new_change_entries(repository_root, arguments.base, arguments.target))
            with open(arguments.out, 'x', encoding='utf-8') as output:
                output.write(content.getvalue())
            print(f'  ✓ 变更清单 {arguments.out}：{entry_total} 条变更记录')
            return 0
        candidates = build_candidates(repository_root, arguments.target, read_fact_table(arguments.facts))
        with open(arguments.out, 'x', encoding='utf-8') as output:
            write_candidates(candidates, output)
        print(f'  ✓ 候选表 {arguments.out}：{len(candidates)} 行（{len({row[3] for row in candidates})} 个不同的行）')
        return 0
    except RecordCarrierRegistryError as error:
        print(f'  ✗ {error}')
        print(f'     → 怎么办：{error.advice}')
        return 2
    except (RuntimeError, ValueError) as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：确认 --base / --target 是这个仓里存在的提交、事实表格式照文件头')
        return 2


BREAK_VARIABLES = ("STALE_CANDIDATES_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行里有 --selftest；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or "--selftest" in arguments or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/kb/stale-candidates.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main())
