#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树）里的三方留痕与同步记录，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 三方论证与阶段同步有没有留下该留的记录：批次范围、判决点名、正文前提、轮次收尾、同步记录
# gate-category: 文档类
# 不声明 gate-covers：.claude/gate-not-implemented.tsv 里的两项（崩溃点重放、模型对拍）判的是被测代码的行为，这一道只判三方论证与阶段同步留下的记录，一项都不覆盖。
#
# 七格，都读 research/prompts/ 与改动范围；每格是一条独立的判据，各有各的出路与计数（原是七道，每道一格，合成这一道）。
# 格名表（--list 打它，--check <格名>[,<格名>…] 只跑点名的格，不给就七格都跑，格名写错退 2）：
# gate-cell: batch-scope 这一批要碰的触发文件有没有登记进范围
# gate-cell: verdict-names-local-samples 新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）
# gate-cell: crates-adversarial-review crates 里的实现改动有没有走过三方正反对抗推理
# gate-cell: implementation-premise 三方论证材料有没有「实现今天的样子」
# gate-cell: abandoned-rounds 发过腿的三方轮次，要么有判决，要么登记撂下
# gate-cell: knowledge-sync 改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录
# gate-cell: agent-def-adversarial-review 改过的 agent 定义与共用约束有没有走过三方审核
# 参数解析、样本根的 .gate-cells、每格一个子 shell、逐格汇总与退出码都经共用库 lib/stage-cells.sh（写法与判据在它的文件头），
# 这里只登记格、解析项目根、在父进程里预取改动范围。每格自己的成功行照旧报它查了多少项。
#
#   bash .claude/gate.d/doc-process-records.sh [项目根]                              七格都跑
#   bash .claude/gate.d/doc-process-records.sh --list                                逐行打格名与判什么，不跑格
#   bash .claude/gate.d/doc-process-records.sh --check <格名>[,<格名>…] [项目根]     只跑点名的格（例：材料员开工只跑 implementation-premise）
#
# 改动范围只经共用库 research/scripts/changed-paths.sh 取，每一种取法这一趟只取一次、各格共用（门禁 code-tooling.sh 的 change-range-single-source 格判阶段里没有第二份取法）：
#   GATE_BASE 起的改动路径（gate 取法：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪都算）
#     ——verdict-names-local-samples 只用其中新增的、crates-adversarial-review、knowledge-sync、agent-def-adversarial-review 用全部；
#   HEAD 起的改动路径（head 取法）——batch-scope；
#   同步记录这一轮新增的行（gate_added_lines，GATE_BASE 起）——knowledge-sync。
# 各格在自己的子 shell 里跑，子 shell 里取的传不回父进程，所以在父进程里按 --check 选中的格预取（样本根的 .gate-cells 在预取之后才生效，
# 那时按全部格预取，多取的几种不用）；取不到时每个要用它的格各自判红、各报各的出路。
# 被判的根不是 git 仓时，用改动范围的五格退 77；implementation-premise 与 abandoned-rounds 只看树，照判。
#
# ── batch-scope ────────────────────────────────────────────────────────────────
# 为什么：knowledge-sync 格的阶段同步按**触发文件**算范围——每多碰一个触发文件，事实表就多一行、候选表多几十到
# 几百行、逐行判与主 agent 的逐行全看跟着涨。而「顺手把踩过的那个脚本修一下」的 diff 只有几行，这个放大器
# 在判「它挡不挡着本轮的课题」的时候看不见。2026-09-22 实测：C468（判错的原始证据不落盘） 本身只要 3 行候选，
# 同一批里顺手修的两个脚本贡献了 249 行候选，逐行判下来 249 行全是「不相干」，多花的挂钟以小时计。
# 写一句「先判阻塞」拦不住手，拦得住的是暂存之后当场红的一道闸（C472（暂存区里多出来的触发文件没人拦））。
#
# 改动范围：**还没进 HEAD 的**——工作区、暂存区与未跟踪文件，基准是 HEAD，不是 GATE_BASE。
# 与 knowledge-sync 格的基准不同是有意的，两者回答的不是同一个问题：
#   knowledge-sync 问「这一轮要回扫哪些」，一轮可以跨几个提交，范围取 GATE_BASE；
#   这一格问「这一次提交要带哪些」，那就只能是还没进 HEAD 的那几个。
# 取 GATE_BASE 会把**上一个提交**的触发文件也算进来（gate-ok 不存在时基准退回 HEAD~1），
# 而那些属于上一批、不在这一批的登记里，每次提交之后必然误红——与判据 ⑤ 躲开的是同一格（C450）。
#
# 判据：
#   ① 触发文件：改动范围里命中 .claude/gate.d/knowledge-sync-triggers.tsv 里任一条正则的路径。
#      那份表是触发文件的唯一登记位，knowledge-sync 格读同一份（表的路径在这个文件里只写一处，两格都从那一处取）；
#      表读不到或一条正则都没有 ⇒ 红。
#   ② 每个触发文件都要在 .claude/batch-scope 里有一行；少一个 ⇒ 红，出路给两条。
#   ③ .claude/batch-scope 里的每个路径都要是这一批的触发文件；多出来的 ⇒ 红
#      （上一批留下的登记会让下一批白放行，判据同 .claude/naming-lint-exclude 的「排除只缩不涨」）。
#   ④ 登记行要写理由：路径之后 # 起，# 后面非空；空理由 ⇒ 红。
#   ⑤ 登记的路径要从仓库根起写（不以 / ./ ../ 开头、不含 /../）；写歪了 ⇒ 红。
#   改动范围一个文件都没有 ⇒ 这一格无对象可判（77，不记通过）。
#
# .claude/batch-scope 的格式：一行一条 <从仓库根起的路径><制表符或空格>#<为什么这一批要碰它>；
# 空行与整行以 # 开头的行是注释。一批提交完把它清空（只留注释）。
#
# 管不到的：理由写得对不对、这一批该不该碰这个文件、登记是不是开工时写的（也可能是判红之后补的）。
# 这几样是语义判断，靠人与 review。
# ⚠️ 几个会话共写一个仓时，不带 --staged 跑它会把**别的会话未提交的改动**也算进这一批，报出一堆不是你的触发文件。
# 判这一批的范围要跑 `bash .claude/scripts/gate.sh --staged`：它在临时 worktree 上只拿 HEAD + 暂存区，
# 那时报出来的就是这一次提交真要带的那几个（`session-wrapup.md` 第 4 条）。
# 判别力：fixtures/doc-process-records.sh/batch-scope-red 放一个没登记的触发文件、一个登记了却不在这一批里的路径、
# 一行没写理由的登记、一个不是从仓库根起的路径，四种必须同时判红；
# batch-scope-green 放三个触发文件（工作区改、暂存新增、未跟踪）都登记了、理由都写了、另有四个不是触发文件的路径不用登记，
# 必须判绿并报对数。
#
# ── verdict-names-local-samples ─────────────────────────────────────────────
# 规则在 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「本地腿交回的每一份样本都读，判决里逐份按路径列出」那一条：
# 干净样本、通读判带损坏的样本、损坏闸留下的作废副本（`-output-void<n>.md`）都要在判决里按路径列出，不许只写「作废」「不稳定不采」。
# 判据（形式）：
#   ① 对象是这一次改动新写的判决 `research/prompts/<轮>-main-verification.md`（只认 research/prompts/ 顶层）。新写的只认三种：
#      相对基准新增（`git diff --diff-filter=A`）、暂存区新增、未跟踪；被改过的旧判决不算、不追溯。
#   ② 轮名取判决文件名去掉 `-main-verification.md`。那一轮本地腿的样本：research/prompts/ 顶层里文件名以 `<轮>-local-` 起头、
#      含 `-output-s` 或 `-output-void`、以 `.md` 结尾的普通文件（含 `<轮>-local-attack-part1-output-s1.md` 这类拆题的）。
#   ③ 每一份样本的文件名都要在那份判决里逐字出现（按文件名字面找，带不带目录都算）；漏的逐个列出，判红。
#   0 字节的样本（ask-local.sh 退 5、退 6 时重定向建出的空 s<n>）不判，成功句逐个报跳过的；那一轮没有本地腿样本的判决不判，照样计数、逐个报出。
#   没有新写的判决 ⇒ 这一格无对象可判（77，不记通过）。
#
# ⚠️ 管不到的：点名了读没读、标得对不对（干净还是参考）、「不稳定」格里的线索有没有逐条写去向——靠人看；
# 文件名出现在「没读」那一句里也算点名（与 crates-adversarial-review、agent-def-adversarial-review 两格同一个盲区）。
# 轮名互为前缀的（`x-r1` 与 `x-r1-local-y-r1`）会把后者的样本算给前者。
#
# 判别力：fixtures/doc-process-records.sh/verdict-names-local-samples-red（两份新写的判决各漏一份作废副本：一份未跟踪、一份暂存新增）与
# verdict-names-local-samples-missing-clean-sample（漏一份干净样本）必须判红并逐个列出漏的；verdict-names-local-samples-green（点名齐、0 字节 s2 没点名、
# 别一轮与子目录里的样本、没有本地腿样本的判决、只补了一句的旧判决）必须判绿并报对数。
# 弄坏开关 VERDICT_LOCAL_SAMPLES_BREAK：=skip-void 不认作废副本（-red 转绿），=require-empty 连 0 字节的也要点名（-green 转红）；
# 带着任一项跑 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d`，那一例判错、自证判红。
#
# ── crates-adversarial-review ───────────────────────────────────────────────
# 本工程的实现流程是三步：写代码 → 多 agent 正反对抗推理（.claude/rules/three-way-inference.md）→ checker / 层 0 / 变异证明会红
# （.claude/rules/implementation-workflow.md）。第二步没有任何东西盯着：代码写完、单测绿、门禁绿，就直接提交了。
# 这一格判的是形式：这次改动里每个 crates/*/src/*.rs 文件，都要在这一次改动新写的某份三方判决文件
# （research/prompts/*-main-verification.md）里被按路径点名。点名了判得对不对，要人看。
# 新写的只认三种：相对基准新增（`git diff --diff-filter=A`）、暂存区新增、未跟踪；被改过的旧判决（路径改写、补一句）不算点名。
# 共用库的 git 调用带 core.quotepath=false——不带的话中文文件名的判决被转成带引号的八进制串，
# 对不上 research/prompts/*-main-verification.md，点名了也算没点。
# 没有 crates 改动 ⇒ 这一格无对象可判（77，show-me-test.md：本次未跑，不记通过）。
# 判别力：fixtures/doc-process-records.sh/crates-adversarial-review-red（一个没人点名、一个只被改过的旧判决点名、一个被中文文件名的新判决点名）
# 必须判红并只列前两个；crates-adversarial-review-green 必须判绿并报对数。
#
# ── implementation-premise ──────────────────────────────────────────────────
# 管的是 `.claude/rules/implementation-first.md` 第 3 条：实现已经在 `crates/` 里（里程碑一 2026-09-14 出口），
# 三方论证的背景材料必须有一行前提写明读过的 `crates/` 路径与看到的事实；没有相关实现时写 grep 命令与零命中。
# 判据只看形式：标题日期在 PREMISE_CUTOFF 及以后的正文（`research/prompts/_*-body.md`）里必须出现 `crates/`。
# 日期取首行（标题）里的第一个 YYYY-MM-DD；标题里没有日期的，证不出它早于 PREMISE_CUTOFF，照要查的算。
# 成功行报查了几份，并逐个列出因为标题日期早于 PREMISE_CUTOFF 而没查的（现算）。
# PREMISE_CUTOFF 取规范落地的次日：2026-09-16 当天那几轮有的写在用户指示之前，那批证据不回改（evidence-discipline「原样保存的证据不许事后改」）。
#
# 另两条，只管标题日期在 SIDE_AND_SNAPSHOT_CUTOFF 及以后的正文（那天起材料员与主 agent 的定义才要求它们，更早的证据不回改）：
#   ② 正文里有一行「本地腿：攻 …」或「本地腿：辩 …」，冒号后面在攻 / 辩之外还写了理由（这一轮本地腿派哪一侧、为什么）；
#   ③ 这一轮有背景材料（`_<轮>-background.md`）的，要有开工快照目录 `<轮>-snapshot/`，小节清单（`_<轮>-checklist.md`）里每个「### 小节清单：`<文件>`」
#      点名的 `.claude/kb/` 文件在快照目录里某一份 sha256 清单里有一行。
# 标题日期早于 PREMISE_CUTOFF 的正文全部不判 ⇒ 这一格无对象可判（77）。
# ⚠️ 它管不到读没读对、改法是不是真按实现写的——那一半靠攻方腿与人。
# 判别力：fixtures/doc-process-records.sh/implementation-premise-red（没有 crates/ 的正文、标题没日期的正文、没写本地腿一侧、缺开工快照）
# 必须判红并逐个点名；implementation-premise-green 必须判绿并报查了几份、没查哪一份。
#
# ── abandoned-rounds ────────────────────────────────────────────────────────
# 实测（2026-09-17 核出）：c245-r3 与 c355-c363-r3 两轮 2026-09-16 发了云端腿提示、本地腿交了样本，
# 云端腿没交、没有判决，之后没人收尾，也没有任何东西报警（里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 33 行）。
#
# 判据（只看 research/prompts/ 顶层文件，不进子目录）：
#   ① 认轮次。文件名是下面五种之一，就算这一轮发过腿；轮名必须以 -r<N> 结尾：
#        _<轮>-body.md          主 agent 写的三方正文（CLAUDE.md 派发表）
#        <轮>-opus.md           云端攻方腿的提示
#        <轮>-sonnet.md         云端正推 / 辩方腿的提示
#        <轮>-local-attack.md   本地攻方腿的提示（.claude/agents/three-way-local-attack.md）
#        <轮>-local-defense.md  本地辩方腿的提示（.claude/agents/three-way-local-defense.md）
#   ② 认判决。两种任一：
#        research/prompts/<轮>-main-verification.md；
#        .claude/kb/ 下某个 .md 的同一小节（一个标题到下一个标题之间）既点名这一轮的材料
#        （出现 research/prompts/<轮>- 或 research/prompts/_<轮>-），又有「**判决」——
#        2026-09-11 有判决文件之前，判决直接写进决策正文或变更史，d16-item1-r2、d26-item4-r1、d26-item5-r4 三轮是这样。
#   ③ 认撂下。research/prompts/abandoned-rounds.tsv 一行一轮，四列用制表符分隔：轮名、日期、为什么撂下、去向。
#   发过腿、没有判决、也没登记 ⇒ 红。登记表写坏（不是四列、有空列、日期不是 YYYY-MM-DD、同一轮登记两行）、
#   登记了一个目录里认不出的轮、登记了已经有判决的轮 ⇒ 也红。没有 research/prompts/、或认不出一个发过腿的轮 ⇒ 这一格无对象可判（77）。
#
# 不判的，成功行逐个列出前缀（现算）：腿提示是旧形态的轮——<题>-local.md、<题>-forward-sonnet.md / -reverse-opus.md
# 这类，或轮名不以 -r<N> 结尾（arm3、d16-items134）。2026-09-17 现查这些文件的首次提交全部在 2026-09-11 或更早，
# 那时还没有判决文件（第一份 2026-09-11 提交），判决直接写进 kb、形态不一，按 ② 认会把「判决」读成任意一句提到材料的话；
# 此后仍用 -local.md 写本地腿提示的轮（到 2026-09-16 为止）都另有 _<轮>-body.md 或 <轮>-opus.md / -sonnet.md，已被 ① 认出；agent 定义要求写成 -local-attack / -local-defense。
#
# ⚠️ 管不到的：判决写得对不对、登记的理由真不真（靠人）；② 的第二种只认「同一小节里既点名又有 **判决」，
# 一段写着「**判决**：没判」的小节也算判过；新轮若既不写正文、腿提示又用了上面五种之外的名字，它落进不判的那一列而不红。
# 归档进版本库的腿文件由 git log 取：不在 git 仓里跑（样本目录）时只看树；在 git 仓里而 git log 失败（对象缺了、仓坏了）判红，
# 不退回只看树——那样归档的轮次整批消失，既没判决也没登记的归档轮不再红。
# 判别力：fixtures/doc-process-records.sh/abandoned-rounds-red 放一轮发了腿没判决没登记、一轮只在 kb 里被提到而没有判决、
# 写坏与说反话的登记（认不出的轮、有判决的轮、日期写坏、少一列、同一轮两行），另由 setup.sh 建一个缺了旧树对象的 git 仓（git log 读不到），必须判红；
# abandoned-rounds-green 放判决文件、kb 小节判决、登记撂下各一轮与三个旧形态前缀，必须判绿并报对数。
#
# ── knowledge-sync ──────────────────────────────────────────────────────────
# 为什么：2026-09-17 一批改动做完后没人回头同步知识——records/2026-09-16-subagent拆分提案.md 与 .claude/agent-common.md
# 里两句「还没用过」在用过之后都留着，另一个会话在那句正下方追加新一批也没改它。用户定：阶段任务结束时有一个同步知识的任务点，
# 派 sweep 做阶段同步，主 agent 判完每处命中写一份同步记录。这一格判的是那份记录的形式。
#
# 判据：
#   ① 触发文件：改动范围里命中 `.claude/gate.d/knowledge-sync-triggers.tsv` 里任一条正则的路径。
#      那份表是触发文件的**唯一登记位**，batch-scope 格读同一份；
#      两格各存一份清单就会判得不一样，一格说要回扫、另一格说不用登记。表读不到或一条正则都没有 ⇒ 红。
#      一个触发文件都没有 ⇒ 这一格无对象可判（77，不记通过）。
#   ② 同步记录：改动范围里的 research/prompts/<阶段>-sync.md（prompts 顶层），且文件里有一行只写 <!-- knowledge-sync -->。
#      每个触发文件都要在某份同步记录里按路径逐字出现（等同 grep -F）；漏的逐个列出 ⇒ 红。
#   ③ 每份同步记录有「## 搜索」小节且不空 ⇒ 否则红。
#   ④ 每份同步记录有「## 命中处置」小节，小节里恰好一张表，表头 | 载体 | 原句 | 处置 |。逐行：
#      载体格是「路径:行号」，路径从仓库根起（可包一层反引号），在仓里存在或在改动范围里；
#      处置格以「改了」「补了」「不改：」之一开头，「不改：」后面要写理由；
#      处置是「改了」「补了」的，载体路径要在改动范围里（记录说改了而文件没动）。任一不合 ⇒ 红。表可以 0 行。
#      这一条判这一轮新写的处置行：新写的记录里每一行，加上基准里就有的旧记录这一轮新加的行（按 gate_added_lines 取的新增行逐字认）；
#      旧记录里早先写下的行点名的是更早一轮改的载体，不判。
#   ⑤ 这一次提交要带上的每份同步记录（**还没进 HEAD** 的那几份，基准与 ①–④ 的 GATE_BASE 不同，理由写在 uncommitted_records 那一段）有「## 原始证据」小节，小节里恰好一张表，表头 | 材料 | 路径 | 行数 |，**至少一行**。
#      逐行：材料格非空；路径格是仓库根起的路径、在仓里现存；行数格是十进制整数，与那份文件的 `wc -l` 逐字相等。任一不合 ⇒ 红。
#      拦的是 C468（判错的原始证据不落盘）：阶段同步判错一行之后，事实表、候选表与逐行判定报告都留在 /tmp，
#      想复盘「当时看得出来吗」一个字节都拿不到，只能由当时判错的人事后重建。⑤ 只对还没进 HEAD 的记录判——
#      已经提交的记录是冻结证据、改不得，它点名的材料也早晚归档进版本库（archive-past-rounds.py 会删掉工作区里的那一份），
#      再判就是判一个不存在的对照。
#
# 同步记录的格式（research/prompts/<阶段>-sync.md）：
#   <!-- knowledge-sync -->
#   # <阶段> 阶段同步
#
#   触发文件：.claude/agents/sweep.md、research/scripts/agent-watch.py（这一阶段改动范围里的触发文件，逐个按路径写全）
#
#   ## 搜索
#   回扫用的每条命令与它的命中计数，例：grep -rn "<旧说法>" .claude records research | wc -l → 3
#
#   ## 命中处置
#   | 载体 | 原句 | 处置 |
#   |---|---|---|
#   | .claude/agent-common.md:120 | <那一行的原句> | 改了：<改成了什么> |
#   | .claude/kb/pitfalls.md:40 | （新增） | 补了：<补了什么> |
#   | records/2026-09-16-subagent拆分提案.md:88 | <那一行的原句> | 不改：<理由，例：说的是那一次发生的事> |
#
#   ## 原始证据
#   | 材料 | 路径 | 行数 |
#   |---|---|---|
#   | 事实表 | research/prompts/<阶段>-facts.tsv | 137 |
#   | 候选表 | research/prompts/<阶段>-candidates.tsv | 138 |
#   | 逐行判定报告（第一段 f1-f11） | research/prompts/<阶段>-judge-f1-f11.md | 420 |
#   三个小节标题逐字写，后面不加字；原句里的 | 写成 \|；代码围栏里的 # 行不算标题。
#
# 管不到的：记录写得对不对（原句是不是那一行、「不改」的理由站不站得住）、回扫搜没搜全、
# 只用过而没改动的东西（用过之后该改的句子，只要没碰触发范围就不触发）——这些靠阶段收尾的任务点与人。
# 「改了」「补了」只核载体文件在改动范围里，不核那一行真的动了；行号不核是否越界；路径里带制表符、换行或引号的，git 仍会转义，认不出。
# ⑤ 只核「点名的那几份在不在、行数对不对得上」：材料的内容对不对、候选表召回全没全、判定报告里那几行判得对不对，一个字都判不了。
# 判别力：fixtures/doc-process-records.sh/knowledge-sync-red 放一个没被点名的触发文件、一个只在不带标记的文件里点名的触发文件、
# 说改了而载体没动、开头不合法、不改没理由、载体没行号、载体不存在、缺「## 搜索」、表头写错，必须判红；
# 另放四种「## 原始证据」的坏形态：整节缺掉、表头写错、表一行都没有、点名的候选表不在仓里、行数与 wc -l 对不上；
# knowledge-sync-green 放三个触发文件分在两份记录里点名、三种处置各一行、中文文件名的载体、原句里的 \|、表后围栏里的竖线行、一份 0 行的表，
# 两份记录各带一张对得上的「## 原始证据」表（含一个反引号包着的路径、一份不以换行结尾的材料），必须判绿并报对数。
#
# ── agent-def-adversarial-review ────────────────────────────────────────────
# 规则在 `.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」：改 `.claude/agents/`、
# `.claude/agent-common.md`、`.claude/main-agent.md` 与改 `crates/` 同规矩，写完要走一轮三方（`.claude/rules/three-way-inference.md`），
# 判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份。形态照 crates-adversarial-review 格。
# 这一格判的是形式：改动范围里每个被管文件，都要在这一次改动新写的某份判决文件里被按路径点名。点名了判得对不对，要人看；
# 新写的只认三种：相对基准新增（`git diff --diff-filter=A`）、暂存区新增、未跟踪；被改过的旧判决（路径改写、补一句）不算点名。
# 与 crates-adversarial-review 格同一个盲区——路径出现在「没攻它」那句里也算点名（`records/2026-09-16-subagent拆分提案.md` 第十一节第二行）。
# 删掉的定义也算改动。没有被管文件的改动 ⇒ 这一格无对象可判（77，不记通过）。
# 用户逐次豁免的那几份：`.claude/agent-def-review-exempt` 一行三段（路径、豁免时那一版的 sha256、为什么），删掉一份定义时第二段写 deleted；
# 按内容哈希认，那份定义再改一个字就不再豁免。
# 判别力：fixtures/doc-process-records.sh/agent-def-adversarial-review-red（共用约束没人点名、主 agent 入口只被旧判决点名、删掉的定义没有去向）
# 必须判红并逐个列出；agent-def-adversarial-review-green（四份被新判决点名、一份删除按 deleted 豁免、顺带改的 README 不算）必须判绿并报对数。
#
# ── 样本 ────────────────────────────────────────────────────────────────────
# fixtures/doc-process-records.sh/ 下每一格一对样本（verdict-names-local-samples 另有一份 -missing-clean-sample），
# 每份样本根放 .gate-cells 只点名它判的那一格，别的格不跑（共用库的样本标记：被判的根不是这道门禁所在的仓时才认）。
# setup.sh 里为别的格补的东西（触发文件清单、同步记录、撂下登记、豁免行、正文里的 crates/）是七格一起跑时留下的，留着不碍事。
# 汇总与 --check 的弄坏开关归共用库：STAGE_CELLS_BREAK=red-swallowed 让红样本整道退 0；STAGE_CELLS_BREAK=marker-ignored 让每份样本七格全跑，
# 汇总那条「跑了 1 格」的 want 找不到。格的判法的弄坏开关写在各格那一段（VERDICT_LOCAL_SAMPLES_BREAK）。
#
# gate-similar: doc-experiments.sh 它的 evidence-in-repo 格也用 gate 取法的改动范围、也读 research/prompts/，但判产物与依据落没落进仓（/tmp 路径、产物新不新），不判三方与同步留下的记录；并进来会让七格之外多一种对象（research/results/ 的产物）
# gate-similar: doc-experiments.sh 它的 archive-past-rounds 格按与 knowledge-sync 格同一个基准删上一轮的实验记录，保留判决与 abandoned-rounds.tsv 供这里读；它判工作区里还留没留旧记录，不判记录写没写、写得全不全
# gate-similar: doc-experiments.sh 它的 verdict-false-named 格名字里也有 verdict，但判的是实验产物判决行里的 false 字段有没有被实验页点名，对象是 research/results/ 与实验页，不是三方判决
# gate-similar: code-tooling.sh 它的 change-range-single-source 格判各阶段取改动范围只经 changed-paths.sh 一份取法；这里是那份取法的使用者之一，不判别的阶段
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# implementation-premise 格按 glob 取正文：没有匹配时展开成空，不留字面的通配串
shopt -s nullglob

STAGE_NAME="$(basename "${BASH_SOURCE[0]}")"
STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$STAGE_DIR/lib/stage-cells.sh"
# 触发文件清单只在这里写一处，batch-scope 与 knowledge-sync 两格都从这里取
TRIGGER_TABLE=".claude/gate.d/knowledge-sync-triggers.tsv"
PREMISE_CUTOFF="2026-09-17"
SIDE_AND_SNAPSHOT_CUTOFF="2026-09-28"

stage_cell batch-scope check_batch_scope "这一批要碰的触发文件有没有登记进范围" \
  "没登记的触发文件先问它挡不挡着本轮课题，挡着的在 .claude/batch-scope 加一行「<从仓库根起的路径>  # 为什么这一批要碰它」，不挡着的挪到下一批；多出来的登记删掉"
stage_cell verdict-names-local-samples check_verdict_names_local_samples "新写的三方判决有没有按路径点名那一轮本地腿的每一份样本（含作废副本）" \
  "逐份读那一轮本地腿交回的样本（干净的、通读判带损坏的、-output-void<n>.md），在新写的判决里按路径列出：.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」"
stage_cell crates-adversarial-review check_crates_adversarial_review "crates 里的实现改动有没有走过三方正反对抗推理" \
  "提交之前走一轮三方（.claude/rules/three-way-inference.md），判决写进 research/prompts/<题>-r<n>-main-verification.md，逐个按路径点名改过的 crates 文件；点名写进新的判决，不往旧判决里补"
stage_cell implementation-premise check_implementation_premise "三方论证材料有没有「实现今天的样子」" \
  "正文前提表里加一行「实现今天的样子」写读过的 crates/ 路径与事实；写明本地腿派哪一侧与理由；补开工快照 research/prompts/<轮>-snapshot/（.claude/agents/three-way-materials.md 第 5b 步）"
stage_cell abandoned-rounds check_abandoned_rounds "发过腿的三方轮次，要么有判决，要么登记撂下" \
  "补判决 research/prompts/<轮>-main-verification.md，或在 research/prompts/abandoned-rounds.tsv 加一行（轮名、日期、为什么撂下、去向，制表符分隔）；登记表写坏的照上面逐行改"
stage_cell knowledge-sync check_knowledge_sync "改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录" \
  "写或补 research/prompts/<阶段>-sync.md（格式见这道门禁文件头 knowledge-sync 那一段）：触发文件逐个按路径点名，「## 搜索」「## 命中处置」「## 原始证据」三节照格式写全"
stage_cell agent-def-adversarial-review check_agent_def_adversarial_review "改过的 agent 定义与共用约束有没有走过三方审核" \
  "走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md 并按路径点名改过的每一份；用户逐次豁免的登记进 .claude/agent-def-review-exempt"
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
        echo "  ✗ 多给了一个参数「$1」（项目根已经是 $root_argument）"
        echo "     → 怎么办：用法是 bash .claude/gate.d/$STAGE_NAME [项目根] [--check <格名>[,<格名>…]]"
        exit 2
      fi
      root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
inside_git=0
git rev-parse --is-inside-work-tree >/dev/null 2>&1 && inside_git=1
LIB_CHANGED_PATHS="$(cd "$STAGE_DIR/../.." && pwd)/research/scripts/changed-paths.sh"
# shellcheck source=../../research/scripts/changed-paths.sh
source "$LIB_CHANGED_PATHS" || { echo "  ✗ 读不到共用库 $LIB_CHANGED_PATHS"; echo "     → 怎么办：改动范围的取法只有那一份，恢复它，别在阶段里另抄一份。"; exit 1; }

# ── 共用的改动范围：每种取法这一趟只取一次，结果留在这几个变量里（在当前 shell 里调，不经 $( ) 子 shell） ──
gate_base=""
head_base=""
changed_since_gate=""; changed_since_gate_loaded=0; changed_since_gate_status=0
added_since_gate=""; added_since_gate_loaded=0; added_since_gate_status=0
changed_since_head=""; changed_since_head_loaded=0; changed_since_head_status=0
added_record_lines=""; added_record_lines_loaded=0; added_record_lines_status=0
if ((inside_git)); then
  gate_base="$(gate_diff_base gate)"
  head_base="$(gate_diff_base head)"
fi

load_changed_since_gate() {
  if ((changed_since_gate_loaded)); then return "$changed_since_gate_status"; fi
  changed_since_gate_loaded=1
  if changed_since_gate="$(gate_changed_paths "$gate_base" untracked)"; then changed_since_gate_status=0; else changed_since_gate_status=1; fi
  return "$changed_since_gate_status"
}
load_added_since_gate() {
  if ((added_since_gate_loaded)); then return "$added_since_gate_status"; fi
  added_since_gate_loaded=1
  if added_since_gate="$(gate_changed_paths "$gate_base" untracked A)"; then added_since_gate_status=0; else added_since_gate_status=1; fi
  return "$added_since_gate_status"
}
load_changed_since_head() {
  if ((changed_since_head_loaded)); then return "$changed_since_head_status"; fi
  changed_since_head_loaded=1
  if changed_since_head="$(gate_changed_paths "$head_base" untracked)"; then changed_since_head_status=0; else changed_since_head_status=1; fi
  return "$changed_since_head_status"
}
load_added_record_lines() {
  if ((added_record_lines_loaded)); then return "$added_record_lines_status"; fi
  added_record_lines_loaded=1
  if added_record_lines="$(gate_added_lines "$gate_base" 'research/prompts/*-sync.md')"; then added_record_lines_status=0; else added_record_lines_status=1; fi
  return "$added_record_lines_status"
}
not_git_repository() {
  echo "  ! $ROOT 不是 git 仓，这一格无对象可判"
  return 77
}

# ── batch-scope ──────────────────────────────────────────────────────────────
check_batch_scope() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_head || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 HEAD）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  # 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - <(printf '%s\n' "$changed_since_head") "$TRIGGER_TABLE" <<'PY'
import os, re, sys

with open(sys.argv[1], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})

TRIGGER_TABLE = sys.argv[2]
SCOPE_FILE = ".claude/batch-scope"

if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，knowledge-sync 格与这一格都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if body and not body.startswith("#"):
        patterns.append(re.compile(body))
if not patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)

if not changed_files:
    print("  ! 相对 HEAD 一个改动都没有，这一格无对象可判")
    sys.exit(77)

triggers = [path for path in changed_files if any(pattern.search(path) for pattern in patterns)]

registered = {}
no_reason, bad_path = [], []
if os.path.isfile(SCOPE_FILE):
    for number, raw in enumerate(open(SCOPE_FILE, encoding="utf-8").read().split("\n"), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        path, _, reason = line.partition("#")
        path = path.strip()
        if not path:
            continue
        registered[path] = number
        if not reason.strip():
            no_reason.append(f"{SCOPE_FILE}:{number}：{path} 后面没写为什么这一批要碰它")
        if path.startswith(("/", "./", "../")) or "/../" in path:
            bad_path.append(f"{SCOPE_FILE}:{number}：{path} 不是从仓库根起写的")

missing = [path for path in triggers if path not in registered]
stale = [path for path in registered if path not in triggers]

failed = False
if missing:
    failed = True
    print(f"  ✗ {len(missing)} 个触发文件没登记进这一批的范围（{SCOPE_FILE} 现在登记了 {len(registered)} 个）：")
    for path in sorted(missing):
        print(f"      {path}")                                  # gate-lint:detail
    print(f"     → 怎么办：每多碰一个触发文件就多一轮阶段同步（knowledge-sync 格按触发文件算范围），所以先问一句"
          "「它挡不挡着这一批的课题」——")
    print(f"               不挡：从暂存区撤出去（git restore --staged <路径>，工作区的改动留着），"
          f"把这件事记进 .claude/kb/checks-owed.md 往后延；")
    print(f"               挡着：在 {SCOPE_FILE} 里补一行「<路径><制表符>#<为什么>」，并认下它带来的那一轮阶段同步。")
if stale:
    failed = True
    print(f"  ✗ {len(stale)} 个登记的路径不是这一批的触发文件（登记项不起作用，会让下一批白放行）：")
    for path in sorted(stale):
        print(f"      {SCOPE_FILE}:{registered[path]}：{path}")  # gate-lint:detail
    print(f"     → 怎么办：上一批提交完没清表就会这样，把这几行删掉；"
          f"路径写错了就照 git status 的写法改（从仓库根起，中文路径不要带引号）。")
if no_reason:
    failed = True
    print(f"  ✗ {len(no_reason)} 行登记没写理由：")
    for entry in no_reason:
        print(f"      {entry}")                                  # gate-lint:detail
    print(f"     → 怎么办：路径后面写 #，# 后面写为什么这一批非碰它不可，理由不许省"
          f"（规矩同 .claude/abbreviations 与 .claude/naming-lint-exclude）。")
if bad_path:
    failed = True
    print(f"  ✗ {len(bad_path)} 行登记的路径不是从仓库根起写的：")
    for entry in bad_path:
        print(f"      {entry}")                                  # gate-lint:detail
    print("     → 怎么办：照 git status 的写法写（例 .claude/gate.d/doc-process-records.sh），别写 ./ 开头或相对别处的路径。")
if failed:
    sys.exit(1)

print(f"  ✓ 这一批的 {len(triggers)} 个触发文件都登记进了 {SCOPE_FILE}，登记的 {len(registered)} 个都在这一批里"
      f"（相对 HEAD 共 {len(changed_files)} 个改动路径，另外 {len(changed_files) - len(triggers)} 个不是触发文件、不用登记；"
      f"触发文件的判据是 {TRIGGER_TABLE} 的 {len(patterns)} 条正则）")
PY
}

# ── verdict-names-local-samples ─────────────────────────────────────────────
check_verdict_names_local_samples() {
  ((inside_git)) || { not_git_repository; return; }
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  local -a new_verdicts=()
  mapfile -t new_verdicts < <(grep -E '^research/prompts/[^/]+-main-verification\.md$' <<<"$added_since_gate" || true)
  if ((${#new_verdicts[@]} == 0)); then
    echo "  ! 这次改动没有新写的判决 research/prompts/<轮>-main-verification.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  python3 - "$gate_base" "${new_verdicts[@]}" <<'PY'
import os
import sys

base = sys.argv[1]
verdict_paths = [path for path in sys.argv[2:] if path]
prompts_directory = "research/prompts"
suffix = "-main-verification.md"
breakage = os.environ.get("VERDICT_LOCAL_SAMPLES_BREAK", "")

top_level_names = sorted(os.listdir(prompts_directory)) if os.path.isdir(prompts_directory) else []


def samples_of(round_name):
    """那一轮本地腿的样本文件名（research/prompts/ 顶层的普通文件）。"""
    prefix = round_name + "-local-"
    found = []
    for name in top_level_names:
        if not (name.startswith(prefix) and name.endswith(".md")):
            continue
        is_void = "-output-void" in name
        if not ("-output-s" in name or is_void):
            continue
        if is_void and breakage == "skip-void":
            continue
        if not os.path.isfile(os.path.join(prompts_directory, name)):
            continue
        found.append(name)
    return found


named_total = 0
with_samples = 0
without_samples = []
skipped_empty = []
missing_by_verdict = []
for verdict_path in verdict_paths:
    if not os.path.isfile(verdict_path):
        continue
    round_name = os.path.basename(verdict_path)[: -len(suffix)]
    samples = samples_of(round_name)
    if not samples:
        without_samples.append(verdict_path)
        continue
    with open(verdict_path, encoding="utf-8", errors="replace") as handle:
        verdict_text = handle.read()
    judged = 0
    missing = []
    for name in samples:
        sample_path = os.path.join(prompts_directory, name)
        if os.path.getsize(sample_path) == 0 and breakage != "require-empty":
            skipped_empty.append(sample_path)
            continue
        judged += 1
        if name in verdict_text:
            named_total += 1
        else:
            missing.append(name)
    if judged:
        with_samples += 1
    else:
        without_samples.append(verdict_path)
    if missing:
        missing_by_verdict.append((verdict_path, missing))

verdict_count = len(verdict_paths)
if missing_by_verdict:
    missing_count = sum(len(missing) for _path, missing in missing_by_verdict)
    print(f"  ✗ 新写的判决里 {len(missing_by_verdict)} 份没按路径点名那一轮本地腿的全部样本（共漏 {missing_count} 份）：")  # gate-lint:summary
    for verdict_path, missing in missing_by_verdict:
        print(f"      {verdict_path} 漏 {len(missing)} 份：")  # gate-lint:detail
        for name in missing:
            print(f"        {name}")  # gate-lint:detail
    print("     → 怎么办：逐份读这一轮本地腿交回的样本（干净的、通读判带损坏的、损坏闸留下的 -output-void<n>.md），在这份判决里按路径列出、")
    print("               各标干净或参考，写它说了什么、去向如何（.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」一节")
    print("               「本地腿交回的每一份样本都读，判决里逐份按路径列出」那一条）；不改样本、不删样本。")
    sys.exit(1)

print(f"  ✓ 新写的判决 {verdict_count} 份：{with_samples} 份逐份点名了那一轮本地腿的 {named_total} 份样本，"
      f"{len(without_samples)} 份那一轮没有本地腿样本；0 字节样本跳过 {len(skipped_empty)} 份（基准 {base}）")
if without_samples:
    print("    没有本地腿样本的：" + "、".join(without_samples))
if skipped_empty:
    print(f"    0 字节样本跳过 {len(skipped_empty)} 份：" + "、".join(skipped_empty))
PY
}

# ── crates-adversarial-review ───────────────────────────────────────────────
check_crates_adversarial_review() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  local sources verdicts source verdict named verdict_count
  local -a missing=()
  local count=0
  sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed_since_gate" || true)"
  if [[ -z "$sources" ]]; then
    echo "  ! 这次改动没碰 crates/*/src，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  verdicts="$(grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added_since_gate" || true)"
  while IFS= read -r source; do
    [[ -z "$source" ]] && continue
    count=$((count + 1))
    named=0
    while IFS= read -r verdict; do
      [[ -z "$verdict" || ! -f "$verdict" ]] && continue
      if grep -qF -- "$source" "$verdict"; then named=1; break; fi
    done <<<"$verdicts"
    ((named)) || missing+=("$source")
  done <<<"$sources"
  if ((${#missing[@]})); then
    echo "  ✗ crates 改动里 ${#missing[@]} 个文件没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）："   # gate-lint:summary
    printf '      %s\n' "${missing[@]}"   # gate-lint:detail
    echo "     → 怎么办：写代码之后、提交之前走一轮三方（.claude/rules/three-way-inference.md）：材料带上 diff，正推 / 反推 / 找反例三条腿各攻一遍，"
    echo "               主 agent 的判决写进 research/prompts/<题>-r<n>-main-verification.md，逐个按路径点名改过的 crates 文件（打中的写回代码再改一轮）；"
    echo "               点名写进新的判决，不往旧判决里补。"
    return 1
  fi
  verdict_count="$(grep -c . <<<"$verdicts" || true)"
  echo "  ✓ crates 改动 ${count} 个文件都有三方判决点名（新写的判决文件 ${verdict_count} 份，基准 $gate_base）"
}

# ── implementation-premise ──────────────────────────────────────────────────
check_implementation_premise() {
  local prompts_directory="research/prompts"
  local body heading date date_label round listed
  local -a bodies=("$prompts_directory"/_*-body.md)
  local checked=0 side_checked=0 failed=0
  local -a missing=() before_cutoff=() undated=() missing_side=() missing_snapshot=()
  for body in "${bodies[@]}"; do
    heading="$(head -n 1 "$body")"
    date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"
    # 标题里没有日期：证不出它早于 PREMISE_CUTOFF，照要查的算，不静默跳过
    if [[ -z "$date" ]]; then
      undated+=("$body")
      date_label="标题没有日期，按要查的算"
    elif [[ "$date" < "$PREMISE_CUTOFF" ]]; then
      before_cutoff+=("$body（$date）")
      continue
    else
      date_label="$date"
    fi
    checked=$((checked+1))
    grep -qF 'crates/' "$body" || missing+=("$body（$date_label）")
    if [[ -n "$date" && ! "$date" < "$SIDE_AND_SNAPSHOT_CUTOFF" ]]; then
      side_checked=$((side_checked + 1))
      grep -qE '本地腿[*]*[：:][*]*[[:space:]]*(攻|辩)[^|]{2,}' "$body" || missing_side+=("$body")
      round="$(basename "$body")"; round="${round#_}"; round="${round%-body.md}"
      if [[ -f "$prompts_directory/_$round-background.md" ]]; then
        if [[ ! -d "$prompts_directory/$round-snapshot" ]]; then
          missing_snapshot+=("$round：没有 research/prompts/$round-snapshot/")
        else
          while IFS= read -r listed; do
            [[ "$listed" == .claude/kb/* ]] || continue
            grep -qsF "$listed" "$prompts_directory/$round-snapshot"/* || missing_snapshot+=("$round：$listed 不在快照清单里")
          done < <(sed -n 's/^### 小节清单：`\([^`]*\)`.*/\1/p' "$prompts_directory/_$round-checklist.md" 2>/dev/null)
        fi
      fi
    fi
  done
  if ((checked == 0)); then
    echo "  ! 标题日期在 $PREMISE_CUTOFF 及以后的三方论证正文 0 份，这一格无对象可判"
    return 77
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 这些三方论证正文没有「实现今天的样子」——全文一处 crates/ 都没有："   # gate-lint:summary
    printf '     %s\n' "${missing[@]}"
    echo "     → 怎么办：读 crates/ 里与这一问相关的代码路径，在正文前提表里加一行「实现今天的样子」，写文件名与看到的事实并标成观测；"
    echo "               实现里没有相关代码时，写 grep 命令与零命中（.claude/rules/implementation-first.md 第 2、3 条）。"
    failed=1
  fi
  if ((${#missing_side[@]})) || ((${#missing_snapshot[@]})); then
    if ((${#missing_side[@]})); then
      echo "  ✗ 这些三方论证正文（标题日期 ≥ $SIDE_AND_SNAPSHOT_CUTOFF）没写这一轮本地腿派哪一侧、为什么："   # gate-lint:summary
      printf '     %s\n' "${missing_side[@]}"   # gate-lint:detail
      echo "     → 怎么办：正文分工表里加一行「本地腿：攻（理由）」或「本地腿：辩（理由）」——辩方复核轮派本地辩方，缺哪一侧派哪一侧（.claude/main-agent.md「什么时候派哪个 agent」）"
    fi
    if ((${#missing_snapshot[@]})); then
      echo "  ✗ 这些轮次的开工快照缺了（有背景材料就要有 <轮>-snapshot/，清单点名的 kb 文件都要在快照清单里）："   # gate-lint:summary
      printf '     %s\n' "${missing_snapshot[@]}"   # gate-lint:detail
      echo "     → 怎么办：照 .claude/agents/three-way-materials.md 第 5b 步补 research/prompts/<轮>-snapshot/kb-sha256.txt（sha256sum 原样输出，路径从仓根起）；腿已经开跑的，快照里写明是补写的与补写的日期"
    fi
    failed=1
  fi
  ((failed)) && return 1
  echo "  ✓ 三方论证正文都写了实现今天的样子（检查了 $checked 份：标题日期 ≥ $PREMISE_CUTOFF 的 $((checked - ${#undated[@]})) 份、标题没有日期的 ${#undated[@]} 份）"
  if ((${#undated[@]})); then
    echo "    标题没有日期、照要查的算的 ${#undated[@]} 份：${undated[*]}"
  fi
  echo "    标题日期 ≥ $SIDE_AND_SNAPSHOT_CUTOFF、另核了本地腿一侧与开工快照的 $side_checked 份"
  if ((${#before_cutoff[@]})); then
    echo "    没查 ${#before_cutoff[@]} 份（标题日期早于 $PREMISE_CUTOFF，规范落地之前写的）：${before_cutoff[*]}"
  else
    echo "    没查 0 份"
  fi
}

# ── abandoned-rounds ────────────────────────────────────────────────────────
check_abandoned_rounds() {
  python3 - <<'PY'
import os, re, subprocess, sys

prompts_dir = "research/prompts"
registry_path = "research/prompts/abandoned-rounds.tsv"
kb_dir = ".claude/kb"

if not os.path.isdir(prompts_dir):
    print(f"  ! 没有 {prompts_dir}，这一格无对象可判")
    sys.exit(77)

round_name = r"(?P<round>[a-z0-9][a-z0-9-]*-r[0-9]+)"
dispatch_forms = (
    re.compile(r"^_" + round_name + r"-body\.md$"),
    re.compile(r"^" + round_name + r"-opus\.md$"),
    re.compile(r"^" + round_name + r"-sonnet\.md$"),
    re.compile(r"^" + round_name + r"-local-attack\.md$"),
    re.compile(r"^" + round_name + r"-local-defense\.md$"),
)
legacy_body_form = re.compile(r"^_(?P<prefix>.+)-body\.md$")
legacy_leg_form = re.compile(r"^(?P<prefix>[^_].*?)(?:(?:-(?:forward|reverse|attack|defense|admission|ledger))?-(?:opus|sonnet)|-local|-local-attack|-local-defense)\.md$")

# 树上现有的，加上按「每一次提交删上一次的实验记录」归档进版本库的那些。
# 少了后一半，一轮的腿文件一被归档，撂下登记表里那一行就报「认不出它发过腿」——
# 而登记表本身是归档规则明令保留的（这一格的输入），两边对不上不是登记错了，是这道检查只看了树。
archived_names = set()
archive_log_error = None                     # 在 git 仓里而 git log 失败：归档的轮次读不到，收尾判红
inside_repository = subprocess.run(["git", "rev-parse", "--is-inside-work-tree"],
                                   capture_output=True, text=True).returncode == 0
if inside_repository:
    log = subprocess.run(["git", "-c", "core.quotepath=false", "log", "--all", "--diff-filter=D", "--format=", "--name-only",
                          "--", prompts_dir], capture_output=True, text=True)
    if log.returncode != 0:
        archive_log_error = f"git log 退出码 {log.returncode}：{(log.stderr.strip().splitlines() or ['（git 没说原因）'])[-1]}"
    else:
        for entry in log.stdout.split("\n"):
            entry = entry.strip()
            if entry.startswith(prompts_dir + "/"):
                archived_names.add(os.path.basename(entry))
# 不在 git 仓里（样本目录）时只看树：那里没有归档可读，判据不变宽
prompt_files = sorted(set(name for name in os.listdir(prompts_dir)
                          if os.path.isfile(os.path.join(prompts_dir, name))) | archived_names)
dispatch_files_by_round = {}
unrecognized_files = []
for name in prompt_files:
    matched_round = None
    for form in dispatch_forms:
        match = form.match(name)
        if match:
            matched_round = match.group("round")
            break
    if matched_round is not None:
        dispatch_files_by_round.setdefault(matched_round, []).append(name)
    elif legacy_body_form.match(name) or legacy_leg_form.match(name):
        unrecognized_files.append(name)

legacy_prefixes = set()
for name in unrecognized_files:
    match = legacy_body_form.match(name) or legacy_leg_form.match(name)
    prefix = match.group("prefix")
    if prefix not in dispatch_files_by_round:
        legacy_prefixes.add(prefix)

if not dispatch_files_by_round:
    print(f"  ! {prompts_dir} 下没有一个文件名认得出发过腿的轮次（_<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md），这一格无对象可判")
    sys.exit(77)

# kb 小节：一个标题行到下一个标题行之间。只读还没有判决文件的轮要用到的那几条。
rounds_without_verdict_file = [name for name in dispatch_files_by_round if not os.path.isfile(os.path.join(prompts_dir, f"{name}-main-verification.md"))]
kb_sections = []
if rounds_without_verdict_file and os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            section_lines = []
            section_heading = ""
            for line in open(path, encoding="utf-8"):
                if re.match(r"^#{1,6} ", line):
                    if section_lines:
                        kb_sections.append((path, section_heading, "".join(section_lines)))
                    section_lines = [line]
                    section_heading = line.strip()
                else:
                    section_lines.append(line)
            if section_lines:
                kb_sections.append((path, section_heading, "".join(section_lines)))

def kb_verdict_location(name):
    citation = re.compile(r"research/prompts/_?" + re.escape(name) + r"-")
    for path, heading, text in kb_sections:
        if citation.search(text) and "**判决" in text:
            return f"{path}「{heading}」"
    return None

verdict_file_rounds = []
kb_verdict_rounds = []
rounds_without_any_verdict = []
for name in sorted(dispatch_files_by_round):
    if name not in rounds_without_verdict_file:
        verdict_file_rounds.append(name)
    elif kb_verdict_location(name):
        kb_verdict_rounds.append(name)
    else:
        rounds_without_any_verdict.append(name)

registered_line_numbers = {}
registered_reasons = {}                      # 轮名 → 第三列「为什么撂下」，判「腿一条没派」那一格要读它
malformed_registry_rows = []
if os.path.isfile(registry_path):
    for line_number, line in enumerate(open(registry_path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4 or any(not field.strip() for field in fields):
            malformed_registry_rows.append(f"第 {line_number} 行不是四列、或有空列：{line}")
            continue
        if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", fields[1].strip()):
            malformed_registry_rows.append(f"第 {line_number} 行日期不是 YYYY-MM-DD：{fields[1]}")
            continue
        registered_line_numbers.setdefault(fields[0].strip(), []).append(line_number)
        if len(fields) >= 3:
            registered_reasons[fields[0].strip()] = fields[2]

duplicated_registrations = [f"{name}（第 {', '.join(map(str, numbers))} 行）" for name, numbers in sorted(registered_line_numbers.items()) if len(numbers) > 1]
# 「腿一条没派」也是合法的撂下理由（正文写完之后现查发现题没了、题面并进别处），
# 这种轮次本来就没有腿文件。它的正文一被归档，这条判据就永远认不出它——而登记表自己那一行
# 写着为什么撂下，比文件名更权威。⇒ 理由里明写没派腿的，不要求认出腿文件。
no_leg_form = re.compile(r"腿一条没派|没有派腿|一条腿都没派")
registered_unknown_rounds = [name for name in sorted(registered_line_numbers)
                             if name not in dispatch_files_by_round
                             and not no_leg_form.search(registered_reasons.get(name, ""))]
registered_rounds_with_verdict = [name for name in sorted(registered_line_numbers) if name in verdict_file_rounds or name in kb_verdict_rounds]
unjudged_unregistered_rounds = [name for name in rounds_without_any_verdict if name not in registered_line_numbers]
abandoned_rounds = [name for name in rounds_without_any_verdict if name in registered_line_numbers]

failed = False
if archive_log_error:
    failed = True
    print(f"  ✗ 归档进版本库的腿文件读不到（{archive_log_error}）：只按树判，已归档、既没判决也没登记的轮次这一轮没判")
    print("     → 怎么办：按 git 的报错修好仓库（git fsck 看缺了哪些对象，从远端或备份找回）再跑；读不到归档不是「没有归档」。")
if unjudged_unregistered_rounds:
    failed = True
    print("  ✗ 这些三方轮次发过腿，却没有判决、也不在撂下登记表里：")  # gate-lint:summary
    for name in unjudged_unregistered_rounds:
        print(f"     {name}（{'、'.join(dispatch_files_by_round[name])}）")  # gate-lint:detail
    print(f"     → 怎么办：补判决（写 {prompts_dir}/<轮>-main-verification.md），或在 {registry_path} 加一行：轮名、日期、为什么撂下、去向（四列，制表符分隔）。")
    print("               腿还在跑的轮次，等腿交齐、判决写完再跑门禁；只在 kb 里提到这一轮而没有「**判决」的小节不算判过。")
if malformed_registry_rows:
    failed = True
    print(f"  ✗ {registry_path} 这些行写坏了：")  # gate-lint:summary
    for entry in malformed_registry_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行四列用制表符分隔：轮名、日期（YYYY-MM-DD）、为什么撂下、去向；四列都要有内容，# 开头的行是注释。")
if duplicated_registrations:
    failed = True
    print("  ✗ 这些轮次在撂下登记表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated_registrations:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，理由与去向写在同一行里。")
if registered_unknown_rounds:
    failed = True
    print(f"  ✗ 撂下登记表登记了这些轮，{prompts_dir} 里认不出它们发过腿（没有 _<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md）：")  # gate-lint:summary
    for name in registered_unknown_rounds:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：轮名多半写错了，改成目录里文件名的前缀；那一轮的文件搬过家就照 .claude/rules/path-moves.md 改；确实没有这一轮就删掉这一行。")
if registered_rounds_with_verdict:
    failed = True
    print("  ✗ 这些轮次登记了撂下，却已经有判决——两处说反话：")  # gate-lint:summary
    for name in registered_rounds_with_verdict:
        where = f"{prompts_dir}/{name}-main-verification.md" if name in verdict_file_rounds else kb_verdict_location(name)
        print(f"     {name}：判决在 {where}")  # gate-lint:detail
    print("     → 怎么办：判决是这一轮的现状，就删掉登记表里那一行；判决那一处其实没判，就把它改成不再写「**判决」或删掉那份判决文件，再留登记。")
if failed:
    sys.exit(1)

checked = len(dispatch_files_by_round)
print(f"  ✓ 发过腿的三方轮次都有判决或撂下登记（查了 {checked} 轮：判决文件 {len(verdict_file_rounds)} 轮、kb 小节里的判决 {len(kb_verdict_rounds)} 轮、登记撂下 {len(abandoned_rounds)} 轮）")
if legacy_prefixes:
    print(f"    没判 {len(legacy_prefixes)} 个前缀（腿提示是旧形态：<题>-local.md、<题>-forward-sonnet.md 这类，或轮名不以 -r<N> 结尾）：{'、'.join(sorted(legacy_prefixes))}")
else:
    print("    没判 0 个前缀")
PY
}

# ── knowledge-sync ──────────────────────────────────────────────────────────
check_knowledge_sync() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  # 同步记录这一轮新增的行（未跟踪的整份算新增）：旧记录里这一轮新加的「改了 / 补了」行也要判载体在不在改动范围里
  load_added_record_lines || {
    echo "  ✗ 取不到同步记录这一轮新增了哪些行（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到新增行时旧记录里新加的处置行没法判，不是通过。"
    return 1
  }
  # 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - "$gate_base" <(printf '%s\n' "$changed_since_gate") <(printf '%s\n' "$added_record_lines") "$TRIGGER_TABLE" "$STAGE_NAME" <<'PY'
import os, re, subprocess, sys

base = sys.argv[1]
with open(sys.argv[2], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})
changed_set = set(changed_files)
# 同步记录这一轮新增的行：{记录路径: {整行}}（gate_added_lines 的「路径<TAB>行文」）
added_lines_by_record = {}
with open(sys.argv[3], encoding="utf-8", errors="replace") as handle:
    for added in handle.read().split("\n"):
        if "\t" in added:
            added_path, added_text = added.split("\t", 1)
            added_lines_by_record.setdefault(added_path, set()).add(added_text)

# 触发文件的清单读 .claude/gate.d/knowledge-sync-triggers.tsv（路径由外层传进来，与 batch-scope 格同一处），不在这里再存一份：
# batch-scope 格拿同一份清单算它那个范围，两格判得不一样谁都说不清该信哪个
# （`show-me-test.md`「两套装置算同一个量，就要有一条检查逼它们落到同一个数」）。
TRIGGER_TABLE = sys.argv[4]
STAGE_NAME = sys.argv[5]
if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，这一格与 batch-scope 格都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
trigger_patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if not body or body.startswith("#"):
        continue
    trigger_patterns.append(re.compile(body))
if not trigger_patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)
trigger_files = [path for path in changed_files if any(pattern.search(path) for pattern in trigger_patterns)]
outside_trigger_count = len(changed_files) - len(trigger_files)
if not trigger_files:
    print(f"  ! 这次改动没碰规则、agent、hook、门禁、脚本与实现，这一格无对象可判（改动范围 {len(changed_files)} 个文件，基准 {base}）")
    sys.exit(77)

marker_line = "<!-- knowledge-sync -->"
record_name = re.compile(r"^research/prompts/[^/]+-sync\.md$")
records = []
fresh_records = set()
uncommitted_records = set()
unmarked_candidates = []
for path in changed_files:
    if not record_name.search(path) or not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    if any(line.strip() == marker_line for line in text.split("\n")):
        records.append((path, text))
        # 这一轮**新写**的那几份：处置行的「改了 / 补了」只对它们判。
        # ⚠️ 跨轮留存的旧记录点名的载体是更早一轮改的，相对今天的基准当然不在范围里，
        # 而那些行说的都是真话——改写它们才是假的（C450 实测 2026-09-21）。
        # 判据用同一个基准：相对基准是新增（A）的才算这一轮写的；只是被改过（M，例如
        # 按 C450 追加一句「为什么这一轮还留着」）的不算。
        # 判据是「基准那一版里有没有这个文件」，不是 `git diff --name-status`：
        # 后者看不见**未跟踪**的新文件，而这一轮刚写出来还没 add 的记录正是那一种
        # （门禁自己的红样本就这么造的，用 diff 判会把它当成旧记录放行）。
        exists_at_base = subprocess.run(
            ["git", "cat-file", "-e", f"{base}:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_base:
            fresh_records.add(path)
        # ⑤ 用的是另一个基准：HEAD，不是 GATE_BASE。两个基准回答的不是同一个问题——
        # 「改了 / 补了」问的是「这一轮的改动范围里有没有这个载体」，范围由 GATE_BASE 定，两边必须同一个基准（C450）；
        # ⑤ 问的是「这一次提交要带上的记录，它的原始材料带了没有」，那就只能是还没进 HEAD 的那几份。
        # 拿 GATE_BASE 判 ⑤ 会连**上一个提交里已经写好的记录**一起判（gate-ok 不存在时基准退回 HEAD~1，
        # 上一个提交新增的记录相对它就是「新写的」），而那份记录是冻结证据、改不得，它的材料也归档了。
        exists_at_head = subprocess.run(
            ["git", "cat-file", "-e", f"HEAD:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_head:
            uncommitted_records.add(path)
    else:
        unmarked_candidates.append(path)

fence_open = re.compile(r"^ {0,3}(`{3,}|~{3,})")
heading = re.compile(r"^#{1,2}\s")


def second_level_sections(text):
    """二级标题整行 → 小节体 [(行号, 行, 是否在代码围栏里)]；同名标题只认第一次，代码围栏里的 # 行不算标题。"""
    sections = {}
    current_body = None
    open_fence = None
    for line_number, line in enumerate(text.split("\n"), 1):
        if open_fence is None:
            fence_match = fence_open.match(line)
            if fence_match:
                open_fence = fence_match.group(1)
                if current_body is not None:
                    current_body.append((line_number, line, True))
                continue
            if heading.match(line):
                title = line.rstrip()
                if title.startswith("## ") and title not in sections:
                    sections[title] = []
                    current_body = sections[title]
                else:
                    current_body = None
                continue
            if current_body is not None:
                current_body.append((line_number, line, False))
        else:
            if re.match(r"^ {0,3}" + re.escape(open_fence[0]) + "{" + str(len(open_fence)) + r",}\s*$", line):
                open_fence = None
            if current_body is not None:
                current_body.append((line_number, line, True))
    return sections


def table_cells(row):
    inner = row.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|") and not inner.endswith("\\|"):
        inner = inner[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", inner)]


carrier_form = re.compile(r"^(?P<path>\S(?:.*\S)?):(?P<line>[1-9][0-9]*)$")
separator_cell = re.compile(r"^:?-{3,}:?$")

missing_triggers = [path for path in trigger_files if not any(path in text for _record_path, text in records)]
search_problems = []
table_problems = []
carrier_problems = []
disposition_start_problems = []
empty_reason_problems = []
out_of_range_problems = []
disposition_counts = {"改了": 0, "补了": 0, "不改": 0}
evidence_problems = []
evidence_rows_total = 0
evidence_record_count = 0


def tables_in(section_body):
    """小节体里的几张表：连着的表格行算一张，代码围栏里的行不算。返回 [[(行号, 整行), …], …]。"""
    runs = []
    previous_was_table_line = False
    for line_number, line, fenced in section_body:
        is_table_line = (not fenced) and line.lstrip().startswith("|")
        if is_table_line and not previous_was_table_line:
            runs.append([])
        if is_table_line:
            runs[-1].append((line_number, line))
        previous_was_table_line = is_table_line
    return runs


def newline_count(path):
    """`wc -l` 报的数：数换行符，不补末尾。不以换行结尾的文件两者差 1，取一个口径，别让写记录的人猜。"""
    with open(path, "rb") as handle:
        return handle.read().count(b"\n")


for record_path, text in records:
    sections = second_level_sections(text)
    if "## 搜索" not in sections:
        search_problems.append(f"{record_path}：没有「## 搜索」小节")
    elif not any(line.strip() for _number, line, _fenced in sections["## 搜索"]):
        search_problems.append(f"{record_path}：「## 搜索」小节是空的")
    # ⑤ 原始证据只对还没进 HEAD 的记录判：已提交的记录改不得，它点名的材料也归档了（C468）。
    if record_path in uncommitted_records:
        evidence_record_count += 1
        if "## 原始证据" not in sections:
            evidence_problems.append(f"{record_path}：没有「## 原始证据」小节")
        else:
            evidence_tables = tables_in(sections["## 原始证据"])
            if not evidence_tables:
                evidence_problems.append(f"{record_path}：「## 原始证据」小节里没有表")
            elif len(evidence_tables) > 1:
                evidence_problems.append(f"{record_path}：「## 原始证据」小节里有 {len(evidence_tables)} 张表，只许一张（第二张从第 {evidence_tables[1][0][0]} 行起）")
            else:
                evidence_rows = evidence_tables[0]
                evidence_header_number, evidence_header_line = evidence_rows[0]
                if table_cells(evidence_header_line) != ["材料", "路径", "行数"]:
                    evidence_problems.append(f"{record_path}:{evidence_header_number}：「## 原始证据」的表头不是 | 材料 | 路径 | 行数 |，实际是 {evidence_header_line.strip()}")
                elif (len(evidence_rows) < 2 or len(table_cells(evidence_rows[1][1])) != 3
                      or not all(separator_cell.match(cell) for cell in table_cells(evidence_rows[1][1]))):
                    evidence_problems.append(f"{record_path}:{evidence_header_number + 1}：「## 原始证据」表头下面一行不是三格的分隔行 |---|---|---|")
                elif len(evidence_rows) < 3:
                    evidence_problems.append(f"{record_path}:{evidence_header_number}：「## 原始证据」的表一行都没有——判错一行之后拿不出任何原始材料，正是这一条要拦的")
                else:
                    for row_number, row_line in evidence_rows[2:]:
                        cells = table_cells(row_line)
                        location = f"{record_path}:{row_number}"
                        evidence_rows_total += 1
                        if len(cells) != 3:
                            evidence_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
                            continue
                        material, evidence_path, declared_lines = cells
                        if len(evidence_path) >= 2 and evidence_path.startswith("`") and evidence_path.endswith("`"):
                            evidence_path = evidence_path[1:-1].strip()
                        if not material:
                            evidence_problems.append(f"{location}：材料格是空的，要写这份材料是什么（事实表 / 候选表 / 逐行判定报告）")
                        if evidence_path.startswith(("/", "./", "../")) or "/../" in evidence_path:
                            evidence_problems.append(f"{location}：路径不是从仓库根起写的：{evidence_path}")
                        elif not os.path.isfile(evidence_path):
                            evidence_problems.append(f"{location}：{evidence_path} 不在仓里——原始材料没入库，判错之后无从复盘")
                        elif not re.fullmatch(r"[0-9]+", declared_lines):
                            evidence_problems.append(f"{location}：行数格不是十进制整数：{declared_lines or '（空）'}")
                        else:
                            actual_lines = newline_count(evidence_path)
                            if actual_lines != int(declared_lines):
                                evidence_problems.append(f"{location}：行数写 {declared_lines}，{evidence_path} 实际 {actual_lines}（wc -l）")
    if "## 命中处置" not in sections:
        table_problems.append(f"{record_path}：没有「## 命中处置」小节")
        continue
    table_runs = tables_in(sections["## 命中处置"])
    if not table_runs:
        table_problems.append(f"{record_path}：「## 命中处置」小节里没有表")
        continue
    if len(table_runs) > 1:
        table_problems.append(f"{record_path}：「## 命中处置」小节里有 {len(table_runs)} 张表，只许一张（第二张从第 {table_runs[1][0][0]} 行起）")
        continue
    table_rows = table_runs[0]
    header_number, header_line = table_rows[0]
    if table_cells(header_line) != ["载体", "原句", "处置"]:
        table_problems.append(f"{record_path}:{header_number}：「## 命中处置」的表头不是 | 载体 | 原句 | 处置 |，实际是 {header_line.strip()}")
        continue
    if len(table_rows) < 2 or len(table_cells(table_rows[1][1])) != 3 or not all(separator_cell.match(cell) for cell in table_cells(table_rows[1][1])):
        table_problems.append(f"{record_path}:{header_number + 1}：表头下面一行不是三格的分隔行 |---|---|---|")
        continue
    for row_number, row_line in table_rows[2:]:
        cells = table_cells(row_line)
        location = f"{record_path}:{row_number}"
        if len(cells) != 3:
            table_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
            continue
        carrier_text, _original_sentence, disposition = cells
        if len(carrier_text) >= 2 and carrier_text.startswith("`") and carrier_text.endswith("`"):
            carrier_text = carrier_text[1:-1].strip()
        carrier_match = carrier_form.match(carrier_text)
        carrier_path = None
        if not carrier_match:
            carrier_problems.append(f"{location}：载体格不是「路径:行号」：{carrier_text}")
        else:
            candidate_path = carrier_match.group("path")
            if candidate_path.startswith(("/", "./", "../")) or "/../" in candidate_path:
                carrier_problems.append(f"{location}：载体路径不是从仓库根起写的：{candidate_path}")
            elif not os.path.exists(candidate_path) and candidate_path not in changed_set:
                carrier_problems.append(f"{location}：载体路径 {candidate_path} 在仓里不存在，也不在改动范围里")
            else:
                carrier_path = candidate_path
        if disposition.startswith("改了") or disposition.startswith("补了"):
            disposition_counts[disposition[:2]] += 1
            # 这一轮写的处置行才判：新写的记录整份都是；旧记录只判这一轮新加的那几行（早先的行点名的是更早一轮改的载体）
            written_this_round = record_path in fresh_records or row_line in added_lines_by_record.get(record_path, set())
            if (written_this_round
                    and carrier_path is not None and carrier_path not in changed_set):
                out_of_range_problems.append(f"{location}：处置写「{disposition[:2]}」，载体 {carrier_path} 不在改动范围里")
        elif disposition.startswith("不改："):
            disposition_counts["不改"] += 1
            if not disposition[len("不改："):].strip():
                empty_reason_problems.append(f"{location}：「不改：」后面没写理由")
        else:
            disposition_start_problems.append(f"{location}：处置开头不合法：{disposition or '（空）'}")

sync_step = ("阶段同步怎么做：阶段任务结束时派 sweep 做阶段同步（sweep 定义里的第四种活），主 agent 判完每处命中写 "
             f"research/prompts/<阶段>-sync.md，格式见 .claude/gate.d/{STAGE_NAME} 文件头「knowledge-sync」那一段。")
failed = False
if missing_triggers:
    failed = True
    print(f"  ✗ {len(missing_triggers)} 个触发文件没在任何同步记录里点名（改动范围里带 <!-- knowledge-sync --> 的 research/prompts/*-sync.md 共 {len(records)} 份，一份都没按路径提到它）：")
    for path in missing_triggers:
        print(f"      {path}")
    if unmarked_candidates:
        print("     这几份文件名像同步记录，但没有只写 <!-- knowledge-sync --> 的那一行，不算：")
        for path in unmarked_candidates:
            print(f"      {path}")
    print("     → 怎么办：这一阶段还没做知识同步就先做；做过的，把上面每个文件按路径写全补进同步记录的「触发文件」一行。")
if search_problems:
    failed = True
    print(f"  ✗ {len(search_problems)} 份同步记录缺「## 搜索」或它是空的：")
    for entry in search_problems:
        print(f"      {entry}")
    print("     → 怎么办：加「## 搜索」小节（标题逐字写、后面不加字），写回扫用的每条命令与它的命中计数。")
if table_problems:
    failed = True
    print(f"  ✗ {len(table_problems)} 处「## 命中处置」的表写坏了：")
    for entry in table_problems:
        print(f"      {entry}")
    print("     → 怎么办：「## 命中处置」下面恰好一张表，表头逐字是 | 载体 | 原句 | 处置 |，第二行 |---|---|---|；每行三格，原句里的 | 写成 \\|；没有命中也留表头（0 行）。")
if carrier_problems:
    failed = True
    print(f"  ✗ {len(carrier_problems)} 行载体格不是仓库根起的「路径:行号」或指不到文件：")
    for entry in carrier_problems:
        print(f"      {entry}")
    print("     → 怎么办：载体格写仓库根起的 路径:行号（例 .claude/agent-common.md:120），路径要在仓里现存；别写成相对 research/prompts/ 的路径。")
if disposition_start_problems:
    failed = True
    print(f"  ✗ {len(disposition_start_problems)} 行处置开头不合法（只认「改了」「补了」「不改：」）：")
    for entry in disposition_start_problems:
        print(f"      {entry}")
    print("     → 怎么办：每处命中先判完再写：改了写「改了：…」，新加的写「补了：…」，不动写「不改：理由」；别写「待定」「看过」。")
if empty_reason_problems:
    failed = True
    print(f"  ✗ {len(empty_reason_problems)} 行「不改：」没写理由：")
    for entry in empty_reason_problems:
        print(f"      {entry}")
    print("     → 怎么办：「不改：」后面写不改的理由，例：冻结证据 / 说的是那一次发生的事 / 同一个词、不同的事。")
if evidence_problems:
    failed = True
    print(f"  ✗ {len(evidence_problems)} 处「## 原始证据」写坏了，或点名的材料不在仓里、行数对不上：")
    for entry in evidence_problems:
        print(f"      {entry}")
    print("     → 怎么办：这一轮新写的同步记录要有「## 原始证据」小节，小节里恰好一张表，表头逐字 | 材料 | 路径 | 行数 |，至少一行；")
    print("               把这一阶段的事实表、候选表与每份逐行判定报告从 /tmp 挪进 research/prompts/ 跟着记录一起提交，行数写 wc -l 报的数。")
if out_of_range_problems:
    failed = True
    print(f"  ✗ {len(out_of_range_problems)} 行处置写「改了」「补了」而载体不在改动范围里（记录说改了，文件没动）：")
    for entry in out_of_range_problems:
        print(f"      {entry}")
    print("     → 怎么办：去把那一处真的改了；判下来不该改的，处置写成「不改：理由」；也对一遍载体路径是不是写成了别的文件。")
if failed:
    print(f"     → {sync_step}")
    sys.exit(1)
hit_rows = sum(disposition_counts.values())
print(f"  ✓ 触发文件 {len(trigger_files)} 个都在同步记录里点名（同步记录 {len(records)} 份；命中 {hit_rows} 行："
      f"改了 {disposition_counts['改了']} / 补了 {disposition_counts['补了']} / 不改 {disposition_counts['不改']}；基准 {base}）；"
      f"这一次提交新带的同步记录 {evidence_record_count} 份，点名的 {evidence_rows_total} 份原始材料都在仓里、行数对得上；"
      f"改动范围里另有 {outside_trigger_count} 个文件不在触发范围")
PY
}

# ── agent-def-adversarial-review ────────────────────────────────────────────
check_agent_def_adversarial_review() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  local definitions verdicts definition verdict named now why verdict_count
  local -a exempted=() missing=()
  local count=0
  definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed_since_gate" || true)"
  if [[ -z "$definitions" ]]; then
    echo "  ! 这次改动没碰 .claude/agents/、.claude/agent-common.md、.claude/main-agent.md，这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  verdicts="$(grep -E '^research/prompts/.*-main-verification\.md$' <<<"$added_since_gate" || true)"
  # 用户逐次豁免的那几份：`.claude/agent-def-review-exempt` 一行三段（路径、豁免时那一版的 sha256、为什么）；豁免的是删掉一份定义时第二段写 deleted。
  # ⚠️ 按**内容哈希**认，不按文件名认：那份定义再改一个字，哈希就对不上、这一格照常红。
  # 只有用户明说「这次不走三方」时才加一行，理由里写明是哪一次、他说了什么。
  local exempt_table=".claude/agent-def-review-exempt"
  while IFS= read -r definition; do
    [[ -z "$definition" ]] && continue
    count=$((count + 1))
    named=0
    while IFS= read -r verdict; do
      [[ -z "$verdict" || ! -f "$verdict" ]] && continue
      if grep -qF -- "$definition" "$verdict"; then named=1; break; fi
    done <<<"$verdicts"
    if ((named)); then continue; fi
    if [[ -f "$exempt_table" ]]; then
      # 删掉的定义没有内容可算哈希：豁免行第二段写 deleted，只在文件确实不在时对得上（文件回来了就按哈希认，deleted 那行不再算）
      if [[ -f "$definition" ]]; then now="$(sha256sum -- "$definition" | cut -d' ' -f1)"; else now="deleted"; fi
      why="$(awk -F'\t' -v path="$definition" -v hash="$now" \
        '$1 !~ /^#/ && NF >= 3 && $1 == path && $2 == hash { print $3; exit }' "$exempt_table")"
      if [[ -n "$why" ]]; then exempted+=("$definition：$why"); continue; fi
    fi
    missing+=("$definition")
  done <<<"$definitions"
  if ((${#missing[@]})); then
    echo "  ✗ 改过的定义与共用约束里 ${#missing[@]} 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没按路径提到它；被改过的旧判决不算）："   # gate-lint:summary
    printf '      %s\n' "${missing[@]}"   # gate-lint:detail
    echo "     → 怎么办：走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，"
    echo "               正文里按路径点名改过的每一份定义（打中的写回定义再改一轮）；点名写进新的判决，不往旧判决里补。"
    return 1
  fi
  verdict_count="$(grep -c . <<<"$verdicts" || true)"
  echo "  ✓ 改过的定义与共用约束 ${count} 份都有去向（新写的判决文件 ${verdict_count} 份，另有 ${#exempted[@]} 份按 $exempt_table 豁免，基准 $gate_base）"
  if ((${#exempted[@]})); then
    echo "    豁免的 ${#exempted[@]} 份（按内容哈希认，那份文件再改一个字就不再豁免）："
    printf '      %s\n' "${exempted[@]}"
  fi
}

# ── 改动范围在父进程里按选中的格预取：各格在自己的子 shell 里跑，子 shell 里取的传不回父进程，也传不到别的格 ──
# 取不到的不在这里判红：状态留在 *_status 里，要用它的格调 load_* 时照样拿到失败，各自判红、各报各的出路。
if ((inside_git)); then
  for cell in "${STAGE_CELLS_SELECTED[@]}"; do
    case "$cell" in
      batch-scope) load_changed_since_head ;;
      verdict-names-local-samples) load_added_since_gate ;;
      crates-adversarial-review|agent-def-adversarial-review) load_changed_since_gate; load_added_since_gate ;;
      knowledge-sync) load_changed_since_gate; load_added_record_lines ;;
    esac
  done
fi

stage_cells_run "$ROOT"
