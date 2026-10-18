# 逐行判 S35（组 F918-F929，共 299 行）

阶段：全仓知识腐烂治理 2026-10-17；基准 ce1457c3，结束在工作区（读文件，没用 git show）。没建副本，没起编译。

## 方法

- 取行：`awk -F'\t' '$1~/^F9(1[89]|2[0-9])$/' candidates.tsv` 得 299 行，各组 62/38/14/49/11/2/5/42/13/24/2/37（F918 到 F929），草稿 `mine.tsv`。
- 定位：每行按原文在工作区文件里核过，299 行的行号都对得上，没有「原文已不在」的。
- 现数：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` = 14；`ls -d crates/*/ | wc -l` = 8（多出 `crates/singlefs-unit-graph`）；`ls .claude/skills` = crash-test、decide、gate、remind；`ls research` = prompts、README.md、results；`ls scripts/*/README.md | wc -l` = 8。
- 对着代码核的：`crash_enumeration_new_pool_file_creation_stream.rs` 的 `FULL_STATES` = 16_777_260、`FULL_STATES_WITH_TWO_STATES_PER_WRITE` = 16_777_240；`crash_enumeration_fixed_script_stream.rs` 的 `FULL_STATES_THROUGH_THE_UNMOUNT` = 1_678_377_204。
- 判法：记录 `records/` 一律按「批次快照 / 那一次发生的事」判事件句不改；现状载体里已经写成新说法的判不相干；现状句里还写旧说法的判要改。简报三版并存之后，引中文节标题的 `briefs/….md` 要指到 `.zh.md`。
- 判定表里「事件句不改」的分句，同一行里的现状分句也逐个核过，没有发现仍写旧的。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F918 | .claude/kb/INDEX.md:26 | 不相干 | 原话「语表：中、英、日三语并排，仓根 READ」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/checks-owed.md:354 | 不相干 | 原话「C528」：说的是上游共享规范的三语仓或上游脚本，不是本仓 README 的语言版本 |
| F918 | .claude/kb/checks-owed.md:640 | 不相干 | 原话「的诊断抢注意力。三语仓（zh / en」：说的是上游共享规范的三语仓或上游脚本，不是本仓 README 的语言版本 |
| F918 | .claude/kb/translation-glossary.md:3 | 不相干 | 原话「iefs/` 的三语版本照这张对照表译。」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/translation-glossary.md:4 | 不相干 | 原话「-text 的 trilingual-glossary」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/translation-glossary.md:8 | 不相干 | 原话「> The trilingual versions」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/translation-glossary.md:9 | 不相干 | 原话「ns. The trilingual-glossary」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/translation-glossary.md:12 | 不相干 | 原话「.claude/trilingual-documents」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/translation-glossary.md:13 | 不相干 | 原话「-text の trilingual-glossary」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | .claude/kb/translation-glossary.md:18 | 不相干 | 原话「<!-- trilingual-glossary」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | CLAUDE.md:49 | 不相干 | 原话「正文、脚本逻辑、三语译文那几行）：不跑」：说的是上游共享规范的三语仓或上游脚本，不是本仓 README 的语言版本 |
| F918 | CLAUDE.md:100 | 不相干 | 原话「；旧的不回头改。三语：`YYYY-MM-」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | CLAUDE.md:101 | 不相干 | 原话「语表：中、英、日三语并排，README」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | README.md:3 | 不相干 | 原话「· [中文](README.zh.md) · [日本」：这一行本身就是三语 README / 简报的新说法（英语默认、中文 .zh.md、日语 .ja.md），与旧事实相反，不受它影响 |
| F918 | records/2026-08-26-文献扫描与第一次实测.md:57 | 不相干 | 原话「**上游**（三语同步，0.0.6 →」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-02-SOP修订轮.md:4 | 不相干 | 原话「.0.22**，三语一起改；项目侧还掉一」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-02-SOP修订轮.md:19 | 不相干 | 原话「补四个术语的三语对应（变异测试 /」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-02-SOP修订轮.md:25 | 不相干 | 原话「## 二、三语与版本」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-02-SOP修订轮.md:29 | 不相干 | 原话「个阶段全绿，含「三语同步」与「规则清单」」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-05-SOP剥离轮.md:4 | 不相干 | 原话「.0.28**，三语一起改；」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-09-D15两格定案.md:74 | 不相干 | 原话「ine.md`，三语同改）」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-16-subagent拆分提案.md:811 | 不相干 | 原话「ety.md` 三语各改三段），三仓自检」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-17-已分配口径三方与两个实验.md:170 | 不相干 | 原话「d` 各加一段，三语文档检查通过。照 C」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-20-rules只写怎么做.md:41 | 不相干 | 原话「0.0.53（三语一起推了）」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-20-rules只写怎么做.md:47 | 不相干 | 原话「- 回扫：三语各 14 份规则、`」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-20-rules只写怎么做.md:48 | 不相干 | 原话「三语章节结构逐份对齐（每」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-20-rules只写怎么做.md:49 | 不相干 | 原话「e` ×3 → 三语 CHANGELOG」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-20-rules只写怎么做.md:50 | 不相干 | 原话「三语门禁全绿（13 /」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-20-rules只写怎么做.md:54 | 不相干 | 原话「同名的案卷目录（三语 15 份 + 本仓」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-21-增补3第4件与checker补三条.md:74 | 不相干 | 原话「实都没毛病。上游三语仓各填了一句把路径规」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-27-验证两档拆分.md:29 | 不相干 | 原话「.md` 一节从三语仓删掉、「门禁管哪一」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-28-agent体系用量时间与调度分析.md:299 | 不相干 | 原话「改动应当罕见」；三语版本每次改规则要一起」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-28-agent体系用量时间与调度分析.md:301 | 不相干 | 原话「话；填之前看上游三语仓 `git sta」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-28-agent体系用量时间与调度分析.md:385 | 不相干 | 原话「- 上游三语仓自己的会话没量。」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-28-门禁59号提速与双机分片.md:92 | 不相干 | 原话「4d 提交之外跑提交时会验的东西」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-09-30-里程碑三开工第一轮.md:1049 | 不相干 | 原话「- 用户改定三处（2026-10」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-01-门禁注释精简.md:25 | 不相干 | 原话「完告诉发版会话；三语译文同步交那边」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-01-门禁注释精简.md:169 | 不相干 | 原话「ripts/` 三语仓逐字相同（`cmp」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑三待优化上下游协调.md:6 | 不相干 | 原话「出口：① 上游三语仓 0.0.64 改」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑三待优化上下游协调.md:13 | 不相干 | 原话「LOSSARY、三语 CHANGELOG」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑三待优化上下游协调.md:15 | 不相干 | 原话「6、8）补正文与三语译文」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑三待优化上下游协调.md:22 | 不相干 | 原话「2 行只在下游。三语都改、盖溯源，三仓门」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑三待优化上下游协调.md:59 | 不相干 | 原话「` 用例形态）、三语 0.0.64 节结」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑五打包与决策进度列.md:224 | 不相干 | 原话「上游三语仓 `scripts」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-11-里程碑五打包与决策进度列.md:226 | 不相干 | 原话「关；规则的除外句三语各补「只改了 `.r」：说的是上游 singlefs-ai-sop 的三语仓与它的发版，不是本仓 README 的语言版本 |
| F918 | records/2026-10-13-三语README与简报.md:1 | 事件句不改 | 原话「# 三语 README 与简」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:5 | 事件句不改 | 原话「riefs 增加三语版本，默认为英语版本」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:10 | 事件句不改 | 原话「c-text 加三语格：登记的文件三份齐」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:11 | 事件句不改 | 原话「ary.md`：三语并排，译文里的术语与」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:17 | 事件句不改 | 原话「oc-text 三语格、样本、归属表行、」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:21 | 事件句不改 | 原话「真仓上跑三语格、照审核清单改译文」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:23 | 事件句不改 | 原话「然后 今天提交 三语的readme 和」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-三语README与简报.md:27 | 事件句不改 | 原话「文原样复制成 `README.zh.md` 与 `br」：这是 2026-10-13 三语 README 与简报那一批的出口与分工表（批次快照）：写的就是那一次把单语拆成三份的事，换成新说法后仍是真话 |
| F918 | records/2026-10-13-复用键与全仓改名.md:39 | 不相干 | 原话「到点的出路句），三语仓同一份脚本、同一句」：说的是上游三语仓（或 4-3 批的脚本改动）里的事，不是本仓 README 是不是中文单语 |
| F918 | records/2026-10-13-复用键与全仓改名.md:42 | 不相干 | 原话「和处置分开」那条三语都改好，标题改成「起」：说的是上游三语仓（或 4-3 批的脚本改动）里的事，不是本仓 README 是不是中文单语 |
| F918 | records/2026-10-13-复用键与全仓改名.md:50 | 不相干 | 原话「与注释纪律的规则三语、上游脚本的整轮改名」：说的是上游三语仓（或 4-3 批的脚本改动）里的事，不是本仓 README 是不是中文单语 |
| F918 | records/2026-10-13-复用键与全仓改名.md:52 | 事件句不改 | 原话「、briefs 三语、records 里」：那一天的交回记录：写的是 README 与简报改三语的提交 f0b94fef、当天文档收尾或改名收尾里碰到 README 三份的事，是事件，不说现状 |
| F918 | records/2026-10-13-复用键与全仓改名.md:53 | 事件句不改 | 原话「ME 与简报改为三语：英语为默认，中文原」：那一天的交回记录：写的是 README 与简报改三语的提交 f0b94fef、当天文档收尾或改名收尾里碰到 README 三份的事，是事件，不说现状 |
| F918 | records/2026-10-13-复用键与全仓改名.md:54 | 事件句不改 | 原话「- 批 3 第三件交回：tool」：那一天的交回记录：写的是 README 与简报改三语的提交 f0b94fef、当天文档收尾或改名收尾里碰到 README 三份的事，是事件，不说现状 |
| F918 | records/2026-10-13-复用键与全仓改名.md:688 | 不相干 | 原话「agent 往三语仓填三问的形态。」：说的是上游三语仓（或 4-3 批的脚本改动）里的事，不是本仓 README 是不是中文单语 |
| F918 | records/2026-10-13-复用键与全仓改名.md:692 | 不相干 | 原话「剪枝要自证」之后三语各填一条「复用也要自」：说的是上游三语仓（或 4-3 批的脚本改动）里的事，不是本仓 README 是不是中文单语 |
| F918 | records/2026-10-13-欠账还五条.md:386 | 不相干 | 原话「上游三语仓没有在飞的发版」：说的是上游三语仓（或 4-3 批的脚本改动）里的事，不是本仓 README 是不是中文单语 |
| F919 | .claude/agents/sweep.md:42 | 不相干 | 原话「冻结证据）与 `briefs/`（按日期不回改）。」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | .claude/kb/INDEX.md:26 | 不相干 | 原话「ADME 与 `briefs/` 的三语版本照它用」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | .claude/kb/checks-owed.md:521 | 不相干 | 原话「C761」：说的是 C761 的去向与 records/briefs 旧句不回改，没有引简报的版本或文件名 |
| F919 | .claude/kb/milestone/02-second-txn.md:5 | 要改 | 原话「briefs/2026-09-28.md」→ 改后：把「briefs/2026-09-28.md「全量（2026-09-30）」一节」改成「briefs/2026-09-28.zh.md「全量（2026-09-30）」一节」，其余各字不动（briefs/2026-09-28.md 现在是英语默认版，该节中文标题「全量（2026-09-30）」只在 .zh.md 里；英语版同节标题是 Full run (2026-09-30)） |
| F919 | .claude/kb/milestone/02-second-txn.md:484 | 要改 | 原话「briefs/2026-09-28.md」→ 改后：把「briefs/2026-09-28.md「全量（2026-09-30）」一节」改成「briefs/2026-09-28.zh.md「全量（2026-09-30）」一节」，其余各字不动（briefs/2026-09-28.md 现在是英语默认版，该节中文标题「全量（2026-09-30）」只在 .zh.md 里；英语版同节标题是 Full run (2026-09-30)） |
| F919 | .claude/kb/milestone/03-third-txn.md:48 | 要改 | 原话「briefs/2026-09-28.md」→ 改后：把「briefs/2026-09-28.md「全量（2026-09-30）」一节」改成「briefs/2026-09-28.zh.md「全量（2026-09-30）」一节」，其余各字不动（briefs/2026-09-28.md 现在是英语默认版，该节中文标题「全量（2026-09-30）」只在 .zh.md 里；英语版同节标题是 Full run (2026-09-30)） |
| F919 | .claude/kb/milestone/03-third-txn.md:111 | 要改 | 原话「里程碑二收尾留下的已知红钉值」→ 改后：把「briefs/2026-10-10.md「这个结果不能读成「验过了、没有红」」一节」改成「briefs/2026-10-10.zh.md「这个结果不能读成「验过了、没有红」」一节」，其余各字不动（中文节标题只在 .zh.md 里，英语版同节是 This result cannot be read as "verified, nothing red"） |
| F919 | .claude/kb/milestone/03-third-txn.md:115 | 不相干 | 原话「一版的简报是 `briefs/2026-09-28」：只引「briefs/2026-09-28.md」这个文件名（现为英语默认版，文件还在），没有带中文节标题，三版并存之后仍指得到 |
| F919 | .claude/kb/milestone/03-third-txn.md:139 | 要改 | 原话「- **仓里已有的**：崩溃点由」→ 改后：把「briefs/2026-09-28.md「全量（2026-09-30）」一节」改成「briefs/2026-09-28.zh.md「全量（2026-09-30）」一节」，其余各字不动（briefs/2026-09-28.md 现在是英语默认版，该节中文标题「全量（2026-09-30）」只在 .zh.md 里；英语版同节标题是 Full run (2026-09-30)） |
| F919 | .claude/kb/milestone/04-fourth-txn.md:72 | 要改 | 原话「briefs/2026-09-28.md」→ 改后：把「briefs/2026-09-28.md「全量（2026-09-30）」一节」改成「briefs/2026-09-28.zh.md「全量（2026-09-30）」一节」，其余各字不动（briefs/2026-09-28.md 现在是英语默认版，该节中文标题「全量（2026-09-30）」只在 .zh.md 里；英语版同节标题是 Full run (2026-09-30)） |
| F919 | .claude/kb/translation-glossary.md:3 | 不相干 | 原话「ADME 与 `briefs/` 的三语版本照这张」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | .claude/kb/translation-glossary.md:8 | 不相干 | 原话「and of `briefs/` are tran」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | .claude/kb/translation-glossary.md:12 | 不相干 | 原话「ADME と `briefs/` の三言語版はこの」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | .claude/kb/translation-glossary.md:29 | 不相干 | 原话「`briefs/` 下按日期一份<b」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | CLAUDE.md:100 | 不相干 | 原话「`briefs/`」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | README.md:64 | 不相干 | 原话「le in [`briefs/`](briefs/」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | README.md:147 | 不相干 | 原话「[`briefs/`](briefs/」：这一行已经是新说法（简报按日期、每份三个版本，或只提 briefs/ 目录），不再说简报是单份中文 |
| F919 | records/2026-09-24-里程碑二收尾调度.md:23 | 事件句不改 | 原话「`briefs/` 当天一份、里程碑」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:46 | 事件句不改 | 原话「流水线 GPU 判器路径跑通；三」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:51 | 事件句不改 | 原话「收尾第 8 步 E152 跑完；」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-09-30-里程碑三开工第一轮.md:1007 | 事件句不改 | 原话「- 改名盘点交回（sweep，o」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-09-30-里程碑三开工第一轮.md:1040 | 事件句不改 | 原话「d`）→ 改 `briefs/` 与 README」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-01-README更新与知识腐烂治理.md:29 | 事件句不改 | 原话「经写了，就是 `briefs/2026-09-28」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-01-删除与并发事务.md:1420 | 事件句不改 | 原话「现数），新写 `briefs/<提交当天日期>.m」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-01-删除与并发事务.md:1527 | 事件句不改 | 原话「DME 并写 `briefs/2026-10-10」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-01-删除与并发事务.md:1572 | 事件句不改 | 原话「号可删）与新 `briefs/2026-10-10」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-01-删除与并发事务.md:1600 | 事件句不改 | 原话「的简报不新开 `briefs/2026-10-11」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-01-删除与并发事务.md:1603 | 事件句不改 | 原话「量，跑完更新 `briefs/2026-10-10」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-11-崩溃放量两段与脚本审计.md:133 | 事件句不改 | 原话「ude/kb/ briefs/ records/`」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-11-崩溃放量两段与脚本审计.md:134 | 事件句不改 | 原话「- 2026-10-13：搬迁补」：记录里那一天的任务、排期与去向（批次快照）：写的是当时怎么派、什么时候写简报，不说今天简报有几个版本 |
| F919 | records/2026-10-11-里程碑三知识腐烂治理.md:23 | 事件句不改 | 原话「成的事（简报 `briefs/2026-10-10」：10-11 腐烂治理那一批的范围说明与分工（批次快照），当时简报还是单份中文；说的是那一次怎么治理，换成新说法这句仍是真话 |
| F919 | records/2026-10-11-里程碑三知识腐烂治理.md:41 | 事件句不改 | 原话「的、各写理由：`briefs/` 三份与 `.cl」：10-11 腐烂治理那一批的范围说明与分工（批次快照），当时简报还是单份中文；说的是那一次怎么治理，换成新说法这句仍是真话 |
| F919 | records/2026-10-11-里程碑三知识腐烂治理.md:109 | 事件句不改 | 原话「不治理的：`briefs/`、`.claude」：10-11 腐烂治理那一批的范围说明与分工（批次快照），当时简报还是单份中文；说的是那一次怎么治理，换成新说法这句仍是真话 |
| F919 | records/2026-10-11-里程碑三知识腐烂治理.md:116 | 事件句不改 | 原话「arch/`、`briefs/` 之外的「编号（旧」：10-11 腐烂治理那一批的范围说明与分工（批次快照），当时简报还是单份中文；说的是那一次怎么治理，换成新说法这句仍是真话 |
| F919 | records/2026-10-13-三语README与简报.md:9 | 事件句不改 | 原话「ADME 与 `briefs/` 下每一份简报各有」：2026-10-13 记录里那一批做了什么（把中文原文挪成 .zh.md、改名文档收尾）：是事件，不说现状 |
| F919 | records/2026-10-13-三语README与简报.md:27 | 事件句不改 | 原话「.md` 与 `briefs/*.zh.md`（复」：2026-10-13 记录里那一批做了什么（把中文原文挪成 .zh.md、改名文档收尾）：是事件，不说现状 |
| F919 | records/2026-10-13-复用键与全仓改名.md:30 | 事件句不改 | 原话「- 2026-10-13：改名文」：2026-10-13 记录里那一批做了什么（把中文原文挪成 .zh.md、改名文档收尾）：是事件，不说现状 |
| F919 | records/2026-10-17-全仓知识腐烂治理.md:17 | 不相干 | 原话「可删的产物）；`briefs/` 与 `.clau」：这一行说 briefs/ 按日期、旧的不回头改，今天仍是真话，与简报几个版本无关 |
| F920 | .claude/agents/kb-scribe.md:33 | 要人看 | 原话「它改写了 `research/**/*.rs` 的」：定义这两句与脚本一致：scripts/kb/relabel-item.py 借用的 .claude/gate.d/lib-item-ref-status.py:75-76 扫描范围仍写 research/**/*.rs，而 research/ 现在只剩 prompts/ 与 results/，实验源码在 scripts/experiments/，那边既不被改写也不被门禁判；要改的是脚本与门禁（归 tooling-writer，仓里只有 records/2026-10-11-里程碑三知识腐烂治理.md 记着，没进 checks-owed），改了再改这句，现在改句子会和脚本对不上 |
| F920 | .claude/agents/kb-scribe.md:42 | 要人看 | 原话「`research/**/*.rs`、`.claude/rules/*.md`」：定义这两句与脚本一致：scripts/kb/relabel-item.py 借用的 .claude/gate.d/lib-item-ref-status.py:75-76 扫描范围仍写 research/**/*.rs，而 research/ 现在只剩 prompts/ 与 results/，实验源码在 scripts/experiments/，那边既不被改写也不被门禁判；要改的是脚本与门禁（归 tooling-writer，仓里只有 records/2026-10-11-里程碑三知识腐烂治理.md 记着，没进 checks-owed），改了再改这句，现在改句子会和脚本对不上 |
| F920 | .claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md:183 | 不相干 | 原话「bcc0d2f:research/e7-index-bench/src/bin/e」：这是 git show dbcc0d2f:research/e7-index-bench/… 取旧探针源码，路径是那个提交里的路径，不是现状里的目录 |
| F920 | .claude/kb/milestone/03-third-txn.md:207 | 事件句不改 | 原话「跑时整体搬家（`research/scripts/` 到 `scri」：说的是 2026-10-11 把 research/scripts 搬进 scripts/ 的那次事故与处置；其中的现状分句逐个核过仍真：scripts/ 有 agents、crash、devices、experiments、gate、hosts、kb、mutation 八个子目录各带 README.md（ls scripts/*/README.md 数出 8），research/ 只剩 prompts、results、README.md，.claude/gate-directory 仍不存在 |
| F920 | .claude/kb/tooling.md:200 | 要改 | 改后的句子：表格里「kb 标记改值后…」那一格改成：kb 标记改值后，`scripts/experiments/**/*.rs` 与 `crates/**/*.rs` 里每一份同名 `const` 都要跟（`research/` 下只剩冻结证据，扫它但由 `.claude/format-constants-exclude` 逐份放行）；`stale=` 列的旧值串不许再出现在 kb 正文（依据 .claude/gate.d/code-source-discipline.sh:435 的 scanned_roots = scripts/experiments、research、crates） |
| F920 | records/2026-09-27-同步远端冲突块.md:322 | 事件句不改 | 原话「子。它改写了 `research/**/*.rs` 的，加」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-09-27-同步远端冲突块.md:330 | 事件句不改 | 原话「子。它改写了 `research/**/*.rs` 的，加」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-09-30-里程碑三开工第一轮.md:1061 | 事件句不改 | 原话「`git mv research/scripts scripts`；」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-10-11-里程碑三待优化上下游协调.md:58 | 事件句不改 | 原话「该会话 回话：`research/scripts` 搬进 `scri」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-10-11-里程碑三知识腐烂治理.md:13 | 事件句不改 | 原话「里：路径搬迁 `research/scripts` → `scrip」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-10-11-里程碑三知识腐烂治理.md:32 | 事件句不改 | 原话「it grep research/scripts` 在 milest」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-10-11-里程碑三知识腐烂治理.md:81 | 事件句不改 | 原话「扫描范围仍是 `research/**/*.rs`，搬到」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-10-11-里程碑三知识腐烂治理.md:107 | 事件句不改 | 原话「扫描范围仍是 `research/**/*.rs`；`im」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F920 | records/2026-10-13-欠账还五条.md:394 | 事件句不改 | 原话「0-11 从 `research/scripts/replay.sh」：records 里那一天的定义原文、搬迁经过或回扫交回（批次快照 / 冻结的冲突块）：写的是当时的范围与搬家，不说今天的目录 |
| F921 | .claude/kb/term-renames.md:99 | 不相干 | 原话「`layer0-shard-configura」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | .claude/kb/term-renames.md:100 | 不相干 | 原话「`layer0-shard-run(?![A-」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | .claude/kb/term-renames.md:103 | 不相干 | 原话「inglefs-layer0-shard(?![A-Za-z」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | .claude/kb/term-renames.md:239 | 不相干 | 原话「`SINGLEFS_LAYER0_THREADS(?」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | .claude/kb/term-renames.md:240 | 不相干 | 原话「`SINGLEFS_LAYER0_FINDINGS_」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | .claude/kb/term-renames.md:241 | 不相干 | 原话「`SINGLEFS_LAYER0_SHARD_PEE」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | .claude/kb/term-renames.md:242 | 不相干 | 原话「`SINGLEFS_LAYER0_SHARD_PEE」：这是 term-renames.md 的改名登记表：旧名（layer0 一类）是正则列、新名在后一列，旧名出现在这里正是它的用途 |
| F921 | records/2026-09-16-subagent拆分提案.md:965 | 事件句不改 | 原话「s/crash/layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-16-subagent拆分提案.md:968 | 事件句不改 | 原话「s/crash/layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-16-subagent拆分提案.md:1162 | 事件句不改 | 原话「路写「改成按 `layer0-shard-configura」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-24-里程碑二收尾调度.md:315 | 事件句不改 | 原话「efs/sop/layer0-shard-rehearsal」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-27-同步远端冲突块.md:1211 | 事件句不改 | 原话「- 崩溃枚举用例：`.claud」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-28-里程碑二收尾接手.md:41 | 事件句不改 | 原话「忽略的配置，`layer0-shard.env` 与 `.」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-28-门禁59号提速与双机分片.md:71 | 事件句不改 | 原话「（8 处）与 `layer0-shard-run.sh`（1」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-09-30-里程碑三开工第一轮.md:1067 | 事件句不改 | 原话「删了），判法 `layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:188 | 事件句不改 | 原话「s/crash/layer0-shard-configura」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:189 | 事件句不改 | 原话「s/crash/layer0-shard-run.sh`」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:242 | 事件句不改 | 原话「- 钩子那一件交回（`/tmp/」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:552 | 事件句不改 | 原话「ch.sh`、`layer0-shard-run.sh` 在」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:558 | 事件句不改 | 原话「bench 与 layer0-shard-run 两行改了理」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:576 | 事件句不改 | 原话「与单机不同）、`layer0-shard-run.sh`（带」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:583 | 事件句不改 | 原话「条路没跟上）；`layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:592 | 事件句不改 | 原话「场 5 格）；`layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:593 | 事件句不改 | 原话「不带前缀起 `layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:595 | 事件句不改 | 原话「条），重起；`layer0-shard-run.sh --」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:597 | 事件句不改 | 原话「- `layer0-shard-run.sh --」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:599 | 事件句不改 | 原话「那一件直接起 `layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:601 | 事件句不改 | 原话「问题进下一批：`layer0-shard-run.sh` 第」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:603 | 事件句不改 | 原话「- 用户定：前面记成「进下一批」」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:625 | 事件句不改 | 原话「带前缀」：是 `layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:633 | 事件句不改 | 原话「二台编译改经 `SINGLEFS_LAYER0_PEER_BUIL」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:638 | 事件句不改 | 原话「mmit`）：`layer0-shard-run.sh --」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:641 | 事件句不改 | 原话「s/crash/layer0-shard-run.sh` 第」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:649 | 事件句不改 | 原话「新的，同时跑 `layer0-shard-run.sh --」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:650 | 事件句不改 | 原话「次。自证本体 `layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-01-对接上游测试脚本规范.md:651 | 事件句不改 | 原话「门狗盯着时跑 `layer0-shard-run.sh --」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-11-scripts审计-第二轮.md:40 | 事件句不改 | 原话「s/crash/layer0-shard-run-selft」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-11-崩溃放量两段与脚本审计.md:51 | 事件句不改 | 原话「- 2026-10-11：课题 」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-11-崩溃放量两段与脚本审计.md:77 | 事件句不改 | 原话「数 32（日志「SINGLEFS_LAYER0_THREADS=3」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-12-里程碑三工具与治理批.md:97 | 事件句不改 | 原话「wo-host-layer0-shard-env、ssh-w」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-12-里程碑三工具与治理批.md:152 | 事件句不改 | 原话「s/crash/layer0-shard-run.sh` →」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-代码欠账批.md:36 | 事件句不改 | 原话「s/crash/layer0-shard-run.sh` →」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-复用键与全仓改名.md:21 | 事件句不改 | 原话「英文标识符改名（用户 10-13」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-复用键与全仓改名.md:51 | 事件句不改 | 原话「- 批 3 两件交回。脚本与门禁」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-复用键与全仓改名.md:54 | 事件句不改 | 原话「- 批 3 第三件交回：tool」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-复用键与全仓改名.md:56 | 事件句不改 | 原话「- 上游 zh 会话发版 0.0」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-欠账还五条.md:75 | 事件句不改 | 原话「light 的 layer0-shard-run.sh --」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-欠账还五条.md:77 | 事件句不改 | 原话「实现员带前缀跑 layer0-shard-run.sh」期望」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F921 | records/2026-10-13-欠账还五条.md:177 | 事件句不改 | 原话「一类也要改名（`layer0-shard-run.sh` →」：records 里那一天的审计、交回、派发或改名通知（批次快照 / 冻结的冲突块）：写的是当时脚本叫什么、发生了什么，不说今天脚本叫什么 |
| F922 | .claude/kb/experiments/116-打包容器的账·补元数据写与整理策略.md:141 | 不相干 | 原话「$ cd research/results &」：「cd research/results」仍对：research/results/ 现在还在（ls research 有 prompts、README.md、results），这里读的是留存产物，不是跑脚本 |
| F922 | .claude/kb/experiments/158-择根与修复四岔路.md:59 | 不相干 | 原话「$ cd research/results;」：「cd research/results」仍对：research/results/ 现在还在（ls research 有 prompts、README.md、results），这里读的是留存产物，不是跑脚本 |
| F922 | .claude/kb/experiments/158-择根与修复四岔路.md:112 | 不相干 | 原话「$ cd research/results;」：「cd research/results」仍对：research/results/ 现在还在（ls research 有 prompts、README.md、results），这里读的是留存产物，不是跑脚本 |
| F922 | .claude/kb/experiments/158-择根与修复四岔路.md:137 | 不相干 | 原话「$ cd research/results;」：「cd research/results」仍对：research/results/ 现在还在（ls research 有 prompts、README.md、results），这里读的是留存产物，不是跑脚本 |
| F922 | .claude/kb/experiments/171-三oracle合谋测试.md:256 | 不相干 | 原话「cd research/results;」：「cd research/results」仍对：research/results/ 现在还在（ls research 有 prompts、README.md、results），这里读的是留存产物，不是跑脚本 |
| F922 | .claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:44 | 要改 | 原话「cd research」→ 改后：把单测与变异那几行的 `cd research` 改成 `cd scripts/experiments`，其后命令里的 `scripts/hosts/…` 改写成 `../hosts/…`、`bash scripts/mutation/mutate.sh <bin> scripts/experiments/<源文件>.rs mutations/<名>.tsv` 改写成 `bash ../mutation/mutate.sh <bin> <源文件>.rs ../mutation/tables/<名>.tsv`（依据 .claude/kb/tooling.md:184 与 scripts/mutation/mutate.sh 文件头「在 scripts/experiments/ 里跑」）；同一代码块里 `cd .. && bash scripts/experiments/replay.sh …` 那行也要核（从 scripts/experiments 起跑要写 `cd ../.. &&` 或直接在仓根跑） |
| F922 | .claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md:47 | 要改 | 原话「cd research」→ 改后：把单测与变异那几行的 `cd research` 改成 `cd scripts/experiments`，其后命令里的 `scripts/hosts/…` 改写成 `../hosts/…`、`bash scripts/mutation/mutate.sh <bin> scripts/experiments/<源文件>.rs mutations/<名>.tsv` 改写成 `bash ../mutation/mutate.sh <bin> <源文件>.rs ../mutation/tables/<名>.tsv`（依据 .claude/kb/tooling.md:184 与 scripts/mutation/mutate.sh 文件头「在 scripts/experiments/ 里跑」）；同一代码块里 `cd .. && bash scripts/experiments/replay.sh …` 那行也要核（从 scripts/experiments 起跑要写 `cd ../.. &&` 或直接在仓根跑） |
| F922 | .claude/kb/experiments/177-逐树不写墓碑的复活数.md:47 | 要改 | 原话「cd research」→ 改后：把单测与变异那几行的 `cd research` 改成 `cd scripts/experiments`，其后命令里的 `scripts/hosts/…` 改写成 `../hosts/…`、`bash scripts/mutation/mutate.sh <bin> scripts/experiments/<源文件>.rs mutations/<名>.tsv` 改写成 `bash ../mutation/mutate.sh <bin> <源文件>.rs ../mutation/tables/<名>.tsv`（依据 .claude/kb/tooling.md:184 与 scripts/mutation/mutate.sh 文件头「在 scripts/experiments/ 里跑」）；同一代码块里 `cd .. && bash scripts/experiments/replay.sh …` 那行也要核（从 scripts/experiments 起跑要写 `cd ../.. &&` 或直接在仓根跑） |
| F922 | .claude/kb/experiments/178-叶容器与记录死亡的计数.md:58 | 要改 | 原话「cd research」→ 改后：把单测与变异那几行的 `cd research` 改成 `cd scripts/experiments`，其后命令里的 `scripts/hosts/…` 改写成 `../hosts/…`、`bash scripts/mutation/mutate.sh <bin> scripts/experiments/<源文件>.rs mutations/<名>.tsv` 改写成 `bash ../mutation/mutate.sh <bin> <源文件>.rs ../mutation/tables/<名>.tsv`（依据 .claude/kb/tooling.md:184 与 scripts/mutation/mutate.sh 文件头「在 scripts/experiments/ 里跑」）；同一代码块里 `cd .. && bash scripts/experiments/replay.sh …` 那行也要核（从 scripts/experiments 起跑要写 `cd ../.. &&` 或直接在仓根跑） |
| F922 | .claude/kb/experiments/95-老化的节点布局臂.md:52 | 要改 | 原话「# research/ 下跑」→ 改后：cargo run --release --bin e95-node-layout-arms          # scripts/experiments/ 下跑 |
| F922 | .claude/kb/vm-harness.md:10 | 要改 | 原话「cd research」→ 改后：```bash 块改成：`cd scripts/experiments`、`cargo build --release --target x86_64-unknown-linux-musl --bin <实验二进制名>`、`bash ../devices/vm-bench.sh target/x86_64-unknown-linux-musl/release/<名> /dev/vda <参数...>`（实验二进制现在编在 scripts/experiments/target/，research/ 下已无 Cargo 工程） |
| F923 | records/2026-09-16-subagent拆分提案.md:282 | 事件句不改 | 原话「ecide` 两个 skill」：records 里 09-16 / 09-27 那一次审计时的 skill 状况（当时只有 gate、decide，crash-test 是桩）：是事件，不说仓里现在有几个 skill；现状句里没有写「三个 skill」的地方（全仓 grep 过） |
| F923 | records/2026-09-27-验证两档拆分.md:44 | 事件句不改 | 原话「的桩，上游删掉那个 skill 之后会指空」：records 里 09-16 / 09-27 那一次审计时的 skill 状况（当时只有 gate、decide，crash-test 是桩）：是事件，不说仓里现在有几个 skill；现状句里没有写「三个 skill」的地方（全仓 grep 过） |
| F924 | .claude/rules/changelog-format.md:12 | 不相干 | 原话「.claude/history-carriers`（一行一个目录或文」：这一行已经是新说法：历史节登记在 .claude/history-carriers，kb-discipline.md 第 8 条对 INDEX.md 不判历史版本节，没有写「唯一不留」 |
| F924 | CLAUDE.md:85 | 不相干 | 原话「.claude/history-carriers`」：这一行已经是新说法：历史节登记在 .claude/history-carriers，kb-discipline.md 第 8 条对 INDEX.md 不判历史版本节，没有写「唯一不留」 |
| F924 | records/2026-09-28-agent体系用量时间与调度分析.md:366 | 事件句不改 | 原话「.claude/history-carriers` 登记表进上游；上」：records 里那一轮做了什么、等什么：179 是 09-28 对 history-carriers 的说法，180 是上游 0.0.63 发版同步，181 是 10-11 治理第二轮改过 CLAUDE.md 那句，都是事件 |
| F924 | records/2026-10-01-对接上游测试脚本规范.md:519 | 事件句不改 | 原话「.claude/history-carriers`（本仓已有）；测试」：records 里那一轮做了什么、等什么：179 是 09-28 对 history-carriers 的说法，180 是上游 0.0.63 发版同步，181 是 10-11 治理第二轮改过 CLAUDE.md 那句，都是事件 |
| F924 | records/2026-10-11-里程碑三知识腐烂治理.md:84 | 事件句不改 | 原话「AUDE.md「唯一不留历史版本节」、`su」：records 里那一轮做了什么、等什么：179 是 09-28 对 history-carriers 的说法，180 是上游 0.0.63 发版同步，181 是 10-11 治理第二轮改过 CLAUDE.md 那句，都是事件 |
| F925 | .claude/kb/INDEX.md:7 | 不相干 | 原话「决策索引：编号、简称、状态、」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/checks-owed.md:187 | 不相干 | 原话「8 在内）只写在决策索引表的一行里：没有字段」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/decisions.md:8 | 不相干 | 原话「## 决策索引」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/decisions.md:10 | 不相干 | 原话「一个文件。** 决策索引表做导航 + 一句话」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/decisions.md:14 | 不相干 | 原话「上限由门禁阶段「决策索引结论列的宽度」（`.」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/decisions.md:28 | 不相干 | 原话「，并自己数一遍与进度列比；decision」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/decisions.md:31 | 不相干 | 原话「# 状态列、进度列与分项清单一起重新生」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/layout/01-first-txn.md:23 | 不相干 | 原话「**与里程碑和决策索引挂钩的形式**（门禁」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/01-first-txn.md:13 | 不相干 | 原话「**与字节表和决策索引挂钩**：每一步有一」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/02-second-txn.md:37 | 不相干 | 原话「**与字节表和决策索引挂钩**：覆盖写、释」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/03-third-txn.md:214 | 不相干 | 原话「四个函数 450 到 1230 」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/08-eighth-txn.md:28 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/09-ninth-txn.md:29 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/11-eleventh-txn.md:28 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/12-twelfth-txn.md:38 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/13-thirteenth-txn.md:3 | 不相干 | 原话「程碑五打包与决策进度列.md` 第七节）；」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/13-thirteenth-txn.md:30 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/14-fourteenth-txn.md:3 | 不相干 | 原话「程碑五打包与决策进度列.md` 第七节）；」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/14-fourteenth-txn.md:28 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/15-fifteenth-txn.md:3 | 不相干 | 原话「程碑五打包与决策进度列.md` 第七节）；」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/15-fifteenth-txn.md:29 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节推的」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/16-sixteenth-txn.md:3 | 不相干 | 原话「程碑五打包与决策进度列.md` 第七节）；」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/16-sixteenth-txn.md:28 | 不相干 | 原话「程碑五打包与决策进度列.md` 第三节把这」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | .claude/kb/milestone/17-seventeenth-txn.md:3 | 不相干 | 原话「程碑五打包与决策进度列.md` 第七节）；」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | CLAUDE.md:74 | 不相干 | 原话「**决策索引**：编号、简称、状」：这一行要么已经带进度列（决策索引五列、门禁比两列、生成器 --write 一起重生成），要么只是提到决策索引或引 records 的节次（第三、七、八节在该记录里都有，grep 过 ^## 标题），不受「索引表没有进度列」影响 |
| F925 | records/2026-08-30-三轮攻击轮.md:91 | 事件句不改 | 原话「第 7 段「**决策索引行的状态 vs 正文」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-09-17-CLAUDE.md去冗余.md:24 | 事件句不改 | 原话「行（格式是软的、决策索引与字节表在哪）」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-10-01-删除与并发事务.md:183 | 事件句不改 | 原话「write`：「决策索引表状态列与正文同步（」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-10-01-删除与并发事务.md:372 | 事件句不改 | 原话「ite`：「✓ 决策索引表状态列与正文同步（」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-10-01-删除与并发事务.md:479 | 事件句不改 | 原话「ite`：「✓ 决策索引表状态列与正文同步（」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-10-01-删除与并发事务.md:580 | 事件句不改 | 原话「ite`：「✓ 决策索引表状态列与正文同步（」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-10-01-删除与并发事务.md:1171 | 事件句不改 | 原话「-write`「决策索引表状态列与正文同步（」：records 里 08-30 到 10-07 某一轮的交回与门禁输出原话（「写回 0 行」是当时那条输出），写的是那一次发生的事 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:1 | 事件句不改 | 原话「打包、决策索引加进度列、剩余功能与里程碑预」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:3 | 事件句不改 | 原话「ions.md 决策索引表格最后增加一列写上」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:8 | 事件句不改 | 原话「2. 决策索引表最后加「进度」列，」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:21 | 事件句不改 | 原话「### 进度列」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:25 | 事件句不改 | 原话「新增 D211 进度列写错一例）、kb-s」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:128 | 事件句不改 | 原话「、F2 索引表加进度列），`--check」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:129 | 事件句不改 | 原话「」改成「状态列、进度列」。门禁 doc-d」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-11-里程碑五打包与决策进度列.md:162 | 事件句不改 | 原话「（497 处，含决策索引与实验索引）；按「紧」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-13-代码欠账批.md:58 | 事件句不改 | 原话「1 行，D2 进度列现为「已实现 12」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F925 | records/2026-10-17-全仓知识腐烂治理.md:50 | 事件句不改 | 原话「、看不到纯新增（进度列这类），记为工具缺口」：10-11 里程碑五打包与进度列那一批的调度记录与后续阶段同步记录（批次快照）：写的是那一次加了进度列、判了几行、有什么缺口，不说今天缺什么 |
| F926 | .claude/agent-common.md:4 | 不相干 | 原话「用约束与各份定义只写怎么做：步骤、约束、判据、」：这一行已经是「只写怎么做」的现行写法（范围含规则、定义、共用约束、skill 正文），或只是引用 rules-discipline.md，不说规则正文只写实测、论证 |
| F926 | .claude/agents/tooling-writer.md:16 | 不相干 | 原话「p/rules/rules-discipline.md`「1. 正文」：这一行已经是「只写怎么做」的现行写法（范围含规则、定义、共用约束、skill 正文），或只是引用 rules-discipline.md，不说规则正文只写实测、论证 |
| F926 | .claude/main-agent.md:3 | 不相干 | 原话「` 进。** 它只写怎么做；为什么这么定、实测」：这一行已经是「只写怎么做」的现行写法（范围含规则、定义、共用约束、skill 正文），或只是引用 rules-discipline.md，不说规则正文只写实测、论证 |
| F926 | .claude/rules/implementation-workflow.md:17 | 不相干 | 原话「ent.md` 只写怎么做——步骤、约束、判据」：这一行已经是「只写怎么做」的现行写法（范围含规则、定义、共用约束、skill 正文），或只是引用 rules-discipline.md，不说规则正文只写实测、论证 |
| F926 | CLAUDE.md:30 | 不相干 | 原话「p/rules/rules-discipline.md」：这一行已经是「只写怎么做」的现行写法（范围含规则、定义、共用约束、skill 正文），或只是引用 rules-discipline.md，不说规则正文只写实测、论证 |
| F926 | CLAUDE.md:68 | 不相干 | 原话「skill 正文只写怎么做（`.claude/」：这一行已经是「只写怎么做」的现行写法（范围含规则、定义、共用约束、skill 正文），或只是引用 rules-discipline.md，不说规则正文只写实测、论证 |
| F926 | records/2026-09-16-subagent拆分提案.md:639 | 事件句不改 | 原话「# 二十八、定义只写怎么做：门禁 71、72」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F926 | records/2026-09-20-rules只写怎么做.md:1 | 事件句不改 | 原话「# 规则只写怎么做：立规矩、立门禁、回」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F926 | records/2026-09-20-rules只写怎么做.md:10 | 事件句不改 | 原话「文档的纪律：正文只写怎么做、做什么不做什么、推」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F926 | records/2026-09-20-rules只写怎么做.md:11 | 事件句不改 | 原话「agent 定义只写怎么做）移植并加了后两条」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F926 | records/2026-09-20-rules只写怎么做.md:43 | 事件句不改 | 原话「：`rules/rules-discipline.md`、`scri」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F926 | records/2026-09-20-rules只写怎么做.md:59 | 事件句不改 | 原话「agent 定义只写怎么做）早就把那一半守住了」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F926 | records/2026-09-28-agent体系用量时间与调度分析.md:272 | 事件句不改 | 原话「rules-discipline」：records 里 09-16 到 09-28 立「只写怎么做」规矩、立 71 号门禁与回扫的经过：写的是那一次的事 |
| F927 | .claude/agent-common.md:56 | 不相干 | 原话「- 重型测试就是 checker」：这一行已经是新说法（--selftest 登记为 light 的改了谁就跑谁，heavy 与没登记的才归提交时；门禁阶段、样本自检、各类 lint 仍只在提交时跑），或说的是仓副本里正文改过的脚本、别的脚本的自检，与「selftest 一律不单跑」无关 |
| F927 | .claude/agent-common.md:92 | 不相干 | 原话「- 扫仓的门禁阶段（本地与共享）」：这一行已经是新说法（--selftest 登记为 light 的改了谁就跑谁，heavy 与没登记的才归提交时；门禁阶段、样本自检、各类 lint 仍只在提交时跑），或说的是仓副本里正文改过的脚本、别的脚本的自检，与「selftest 一律不单跑」无关 |
| F927 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1359 | 事件句不改 | 原话「- v1824（tsv-1823」：E165 那一轮的交回说明：这次没跑哪些自证、变异整表归提交时，是那一次的事 |
| F927 | .claude/rules/implementation-workflow.md:46 | 不相干 | 原话「仓副本里正文改过的脚本的自证」：这一行已经是新说法（--selftest 登记为 light 的改了谁就跑谁，heavy 与没登记的才归提交时；门禁阶段、样本自检、各类 lint 仍只在提交时跑），或说的是仓副本里正文改过的脚本、别的脚本的自检，与「selftest 一律不单跑」无关 |
| F927 | .claude/rules/implementation-workflow.md:68 | 不相干 | 原话「**门禁管哪一半**：崩溃点重放」：这一行已经是新说法（--selftest 登记为 light 的改了谁就跑谁，heavy 与没登记的才归提交时；门禁阶段、样本自检、各类 lint 仍只在提交时跑），或说的是仓副本里正文改过的脚本、别的脚本的自检，与「selftest 一律不单跑」无关 |
| F927 | .claude/skills/gate/SKILL.md:12 | 不相干 | 原话「`gate.sh` 与 `che」：这一行已经是新说法（--selftest 登记为 light 的改了谁就跑谁，heavy 与没登记的才归提交时；门禁阶段、样本自检、各类 lint 仍只在提交时跑），或说的是仓副本里正文改过的脚本、别的脚本的自检，与「selftest 一律不单跑」无关 |
| F927 | records/2026-09-16-subagent拆分提案.md:950 | 事件句不改 | 原话「门禁 54、55、57、59 号」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-09-16-subagent拆分提案.md:1154 | 事件句不改 | 原话「- 资源报告与提醒钩子交回（工具」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-09-24-里程碑二收尾调度.md:173 | 事件句不改 | 原话「门禁 54、55、57、59 接」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-09-24-里程碑二收尾调度.md:181 | 事件句不改 | 原话「第十步素材（2026-09-26」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-09-28-门禁59号提速与双机分片.md:95 | 事件句不改 | 原话「4g 收尾两件」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-09-30-里程碑三开工第一轮.md:959 | 事件句不改 | 原话「- 实现员定义第 17 行指针 」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-09-30-里程碑三开工第一轮.md:975 | 事件句不改 | 原话「- G2 交回（implemen」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-对接上游测试脚本规范.md:142 | 事件句不改 | 原话「看门狗记录」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:80 | 事件句不改 | 原话「- E165 对抗一族入库与第二」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:157 | 事件句不改 | 原话「- 代码轮攻方腿交回：`m3-v」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:217 | 事件句不改 | 原话「- 3-7 交回（报告 `/tm」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:235 | 事件句不改 | 原话「- 3-8 交回（报告 `/tm」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:676 | 事件句不改 | 原话「提交时的待办（事实表第八节 45」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:1308 | 事件句不改 | 原话「- E171 比对器补量交回（`」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-里程碑三形式化接入.md:1366 | 事件句不改 | 原话「- 38-2 交回（报告 `~/」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-01-门禁注释精简.md:45 | 事件句不改 | 原话「- 簇 E 交回（报告 `/tm」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-11-scripts审计.md:3 | 事件句不改 | 原话「范围：`scripts/` 下 」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F927 | records/2026-10-13-复用键与全仓改名.md:650 | 事件句不改 | 原话「4-3g 交回（`~/.loca」：records 里那一天的交回、派发、审计或待办（批次快照）：写的是当时谁跑了或没跑哪条自证、哪条归提交时，不说今天 --selftest 的规矩 |
| F928 | .claude/rules/implementation-workflow.md:43 | 不相干 | 原话「的副本里跑；重型一样都不跑（三条不带 `--q」：这一行已经是新说法：子 agent 不跑重型、轻型只跑形式化那一条，崩溃与变异两条轻型的命令写进报告交主 agent |
| F928 | .claude/skills/crash-test/SKILL.md:41 | 不相干 | 原话「nt 不跑重型，轻型只跑形式化那一条，崩溃与变异两」：这一行已经是新说法：子 agent 不跑重型、轻型只跑形式化那一条，崩溃与变异两条轻型的命令写进报告交主 agent |
| F929 | .claude/kb/checks-owed.md:675 | 事件句不改 | 原话「eam.rs`，262165 个状态与产物计数逐」：登记行末格是 2026-09-14，说的是那一天定案时崩溃点重放全量当时的状态数与违例数，是事件 |
| F929 | .claude/kb/checks-owed.md:677 | 事件句不改 | 原话「崩溃点重放全量 262165 个状态违例 2 →」：登记行末格是 2026-09-14，说的是那一天定案时崩溃点重放全量当时的状态数与违例数，是事件 |
| F929 | .claude/kb/experiments.md:164 | 事件句不改 | 原话「E142（新池新建文件的干跑）」：这一句自己写明是「旧 8 单元布局下的判决…见实验页历史版本」，说的是旧布局当时的数 |
| F929 | .claude/kb/experiments/142-新池新建文件的干跑.md:21 | 事件句不改 | 原话「崩溃点重放主臂 262165 个状态零违例（枚举」：E142 页里第七次跑 / 第十七次跑的读数与跑前登记的旧几何（8 单元、33 次写）：说的是那几次跑出的数，不是今天新池新建文件流的状态数 |
| F929 | .claude/kb/experiments/142-新池新建文件的干跑.md:189 | 事件句不改 | 原话「崩溃点重放主臂（问题单第 7 行」：E142 页里第七次跑 / 第十七次跑的读数与跑前登记的旧几何（8 单元、33 次写）：说的是那几次跑出的数，不是今天新池新建文件流的状态数 |
| F929 | .claude/kb/experiments/142-新池新建文件的干跑.md:193 | 事件句不改 | 原话「还留着的旧字面（262165 个状态那一批里数出」：E142 页里第七次跑 / 第十七次跑的读数与跑前登记的旧几何（8 单元、33 次写）：说的是那几次跑出的数，不是今天新池新建文件流的状态数 |
| F929 | .claude/kb/experiments/142-新池新建文件的干跑.md:309 | 要人看 | 原话「重放全量枚举的 262165 个自然状态里这两格」：「262165 个自然状态里这两格恒为 0」是 E142 第十四次跑（2026-09-25）的读数，现在写成现状局限的支撑数；新池新建文件流现在 16 777 260 个状态，装置输出里 header311_reader_branch_counts 近几次是 skipped=true，crates/ 里也没有这个计数的钉值，没法判这个数今天还对不对，该不该加「第十四次跑时」要主 agent 定 |
| F929 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:5 | 不相干 | 原话「量」按字面成了 16 777 260 个状态。重定取样域」：这一行已经是新说法（新池新建文件流 16 777 260 个状态、每次写只取两态是 16 777 240、11 段） |
| F929 | .claude/kb/milestone/01-first-txn.md:159 | 事件句不改 | 原话「**现状（2026-09-14 」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/01-first-txn.md:217 | 事件句不改 | 原话「许失败）。全量 262165 个状态 = 闭式，」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/01-first-txn.md:224 | 事件句不改 | 原话「跑崩溃点重放是 262165 个状态（暖机第二次」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:57 | 事件句不改 | 原话「放全量状态数仍 262165，计数与新产物逐字比」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:64 | 事件句不改 | 原话「states=262165`、五条段序列与第七」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:68 | 事件句不改 | 原话「9 次写、闭式 2104413 个状态（八个 18」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:198 | 事件句不改 | 原话「**现状（2026-09-17 」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:225 | 事件句不改 | 原话「次写、全量枚举 1 678 377 204 个状态（`FULL」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:227 | 事件句不改 | 原话「新池新建文件流 262165 个状态零违例（20」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/02-second-txn.md:484 | 事件句不改 | 原话「次写、全量枚举 1 678 377 204 个状态（`FULL」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/03-third-txn.md:48 | 事件句不改 | 原话「次写、全量枚举 1 678 377 204 个状态（`FULL」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/milestone/04-fourth-txn.md:72 | 事件句不改 | 原话「次写、全量枚举 1 678 377 204 个状态（`FULL」：里程碑文件里某一步落地时的复核清单、带日期的「现状」块或某一次跑的产物读数（2026-09-14 到 2026-09-30）：说的是那个日期那一步落地时的状态数；后来改动已由同段 2026-10-11 注或同表改名行覆盖，这一句不说今天的数 |
| F929 | .claude/kb/verification-build.md:4 | 要改 | 原话「crates/` 下七个 crate」→ 改后：两处要改：一、「`crates/` 下七个 crate」改成「`crates/` 下八个 crate」，并在 `singlefs-reference-filesystem` 之后、`singlefs-harness` 之前补一项「`singlefs-unit-graph`（函数单元图：Rust 与 WGSL 源码按条目切成单元、取去注释指纹，从入口收闭包、算复用键，只用 std，不依赖工作区任何 crate，`singlefs-checker-tier` 依赖它）」（ls -d crates/*/ 数行得 8，crates/singlefs-unit-graph/Cargo.toml 的 description）；二、「登记的崩溃枚举用例 13 条」改成「登记的崩溃枚举用例 14 条」（grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv 数出 14）。同一数在 CLAUDE.md:8 也写「七个 crate」，一起改 |
| F929 | .claude/kb/verification-build.md:128 | 不相干 | 原话「子集，状态数是 16 777 260（`crash_en」：这一行已经是新说法（新池新建文件流 16 777 260 个状态、每次写只取两态是 16 777 240、11 段） |
| F929 | records/2026-09-13-总审核.md:375 | 事件句不改 | 原话「262165 个崩溃状态怎么物化」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-13-总审核.md:401 | 事件句不改 | 原话「同，崩溃点重放 262165 个状态零违例，25」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-13-总审核.md:421 | 事件句不改 | 原话「节探针）。全量 262165 个状态零违例，十个」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-13-总审核.md:449 | 事件句不改 | 原话「点重放全量**（262165 个状态，relea」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-13-总审核.md:454 | 事件句不改 | 原话「崩溃点重放全量 262165 个状态里 2618」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-13-总审核.md:467 | 事件句不改 | 原话「崩溃点重放全量 262165 个状态 check」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-21-增补3第4件与checker补三条.md:31 | 事件句不改 | 原话「release）262165 状态零违例。」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-21-增补3第4件与checker补三条.md:40 | 事件句不改 | 原话「崩溃点重放全量 262165 个状态零违例，这个」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-24-里程碑二收尾调度.md:161 | 事件句不改 | 原话「仍是 18 与 262165，`new_pool」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-24-里程碑二收尾调度.md:163 | 事件句不改 | 原话「崩溃点重放规模与 E142 第二」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:38 | 事件句不改 | 原话「捶打：checker 档快档在 」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-10-01-删除与并发事务.md:450 | 事件句不改 | 原话「564 变成 1 678 377 204（`scripts/」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | records/2026-10-11-里程碑三知识腐烂治理.md:72 | 事件句不改 | 原话「式，全量枚举是 1 678 377 204（`crash_en」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | crates/singlefs-checker/invariants/02-根环.md:14 | 事件句不改 | 原话「I-7.7」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |
| F929 | crates/singlefs-checker/invariants/02-根环.md:15 | 事件句不改 | 原话「I-7.8」：records 里 09-13 到 10-11 的审核、交回与调度记录，或带 2026-09-14 定案日期的不变量说明（I-7.7、I-7.8 的判法由来）：说的是那时崩溃点重放全量的数与违例，不说今天的状态数 |

## 要改的清单（给主 agent 或 kb-scribe）

- 简报引用（5 行同一改法，加 1 行）：`.claude/kb/milestone/02-second-txn.md:5`、`02-second-txn.md:484`、`03-third-txn.md:48`、`03-third-txn.md:139`、`04-fourth-txn.md:72` 里 `briefs/2026-09-28.md「全量（2026-09-30）」一节` 改成 `briefs/2026-09-28.zh.md「全量（2026-09-30）」一节`；`03-third-txn.md:111` 里 `briefs/2026-10-10.md「这个结果不能读成「验过了、没有红」」一节` 改成 `briefs/2026-10-10.zh.md …`（中文节标题只在 `.zh.md` 里，英语版同节是 `Full run (2026-09-30)` 与 `This result cannot be read as "verified, nothing red"`）。`03-third-txn.md:115` 和 `02-second-txn.md:5` 里不带节标题的 `briefs/2026-09-28.md` 不用动。
- `.claude/kb/tooling.md:200`：format-constants 格那一行写 `research/**/*.rs`，格实际扫 `scripts/experiments/`、`research/`、`crates/`（`.claude/gate.d/code-source-discipline.sh:435`），改后的句子见表。
- `.claude/kb/experiments/173、176、177、178` 四页的复跑块里 `cd research`，加 `95-老化的节点布局臂.md:52` 的注释 `# research/ 下跑`，加 `.claude/kb/vm-harness.md:10` 的 `cd research`：`research/` 下已没有 Cargo 工程，改成 `scripts/experiments`（其后相对路径改 `../hosts/…`、`../mutation/…`、`../devices/…`），写法对照 `.claude/kb/tooling.md:184`。这些块里 `cd .. && bash scripts/experiments/replay.sh …` 那几行跟着一起核。
- `.claude/kb/verification-build.md:4`：七个 crate 改八个并补 `singlefs-unit-graph`；登记的崩溃枚举用例 13 条改 14 条。

## 要主 agent 现查

- `.claude/agents/kb-scribe.md:33`、`:42`（两行，要人看）：句子和脚本一致，毛病在脚本：`.claude/gate.d/lib-item-ref-status.py:75-76` 的扫描范围仍是 `research/**/*.rs`，实验源码已搬到 `scripts/experiments/`，那里的分项引用既不被 `scripts/kb/relabel-item.py` 改写，也不被门禁判。`records/2026-10-11-里程碑三知识腐烂治理.md` 记过一次「下一批治理交 tooling-writer」，kb 的 checks-owed 里没有这笔账。要不要登记欠账、谁改脚本，主 agent 定；定义那两句等脚本改了再改。
- `.claude/kb/experiments/142-新池新建文件的干跑.md:309`（要人看）：局限第 9 条把第十四次跑的读数「262165 个自然状态里这两格恒为 0」写成现状支撑数。装置的 `header311_reader_branch_counts` 近几次输出是 skipped=true，`crates/` 里没有同名计数，我判不了这个数今天还对不对，也判不了是否加「第十四次跑时」。

## 工作单之外顺手看到的（候选表里没有这几行）

- `CLAUDE.md:8` 也写「代码在 `crates/` 下七个 crate」，列了七种，漏 `singlefs-unit-graph`，应与 `verification-build.md:4` 同改。
- README 三语的 Layout 表（`README.md`、`README.zh.md`、`README.ja.md` 各自第 128 到 138 行一带）列的 `crates/` 也没有 `crates/singlefs-unit-graph`；要补就三处一起写。
- `crates/singlefs-unit-graph` 在 `git log` 里没有提交记录（可能还在暂存区或工作区），所以上面两处是否这一批就改，由主 agent 看这个 crate 是不是随这次提交进库。

## 汇总

- 299 行：要改 14、要补 0、事件句不改 164、不相干 118、要人看 3。
- 14 行要改里：简报引用 6、tooling.md 1、实验页与 vm-harness 的 `cd research` 6（173、176、177、178 四页，95 页，vm-harness）、verification-build.md 1。
- 同一载体行被几个组命中的（如 `.claude/kb/milestone/03-third-txn.md:48`、`CLAUDE.md:100`、`.claude/kb/translation-glossary.md:3`）每组各写一行相同判定。
- 没建副本，没起编译，没改任何仓内文件。

## 核报告

`python3 scripts/kb/stale-candidates.py --check-report candidates.tsv judge-S35.md --groups F918,F919,F920,F921,F922,F923,F924,F925,F926,F927,F928,F929` 退出码 0，末行：

✓ 报告判全了：299 行候选都有逐行判定
