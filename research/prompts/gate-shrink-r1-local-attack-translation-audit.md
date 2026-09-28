# gate-shrink-r1 本地攻方提示 逐句核转述

核对对象：`research/prompts/gate-shrink-r1-local-attack.md`（英文提示，交给本地模型）。
下表每行：英文项（提示里的原句，按表格与行号定位，不摘句改写）/ 原文文件:行 / 首稿缺的 /
定稿。原文行号在 `research/prompts/gate-shrink-r1-inventory/cells.tsv` 里现查
（该文件第 1 行是表头，因此第 N 格对应该文件第 N+1 行；下面「原文文件:行」栏直接写文件真实行号，
已用 `grep -n` 现取，非从背景材料数）。逐句核法：把 cells.tsv 第 4 列（判什么）原文按分句与英文
译句配对，缺一个限定词就在「首稿缺的」写出来、在「定稿」里补回；英文比中文多出来的限定词或括注
同样在「首稿缺的」写「无」、在「定稿」写多出来的部分与为什么加。

## 一、Q1 逐格映射：84 格「判什么」一列的转述

| 提示里定位 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| T1 row 1（experiment-refs） | cells.tsv:2 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T1 row 2（decision-refs） | cells.tsv:3 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T1 row 3（declared-counts） | cells.tsv:4 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T1 row 4（governance-refs） | cells.tsv:5 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T1 row 5（invariant-count-elsewhere） | cells.tsv:6 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T1 row 6（citations） | cells.tsv:7 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T2 row 1（invariant-anchors） | cells.tsv:8 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T2 row 2（batch-scope） | cells.tsv:9 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T2 row 3（verdict-names-local-samples） | cells.tsv:10 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T2 row 4（crates-adversarial-review） | cells.tsv:11 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T2 row 5（implementation-premise） | cells.tsv:12 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T2 row 6（abandoned-rounds） | cells.tsv:13 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T3 row 1（knowledge-sync） | cells.tsv:14 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T3 row 2（agent-def-adversarial-review） | cells.tsv:15 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T3 row 3（prime-marks） | cells.tsv:16 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T3 row 4（clock-times） | cells.tsv:17 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T3 row 5（term-renames） | cells.tsv:18 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T3 row 6（vague-names） | cells.tsv:19 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T4 row 1（test-file-names） | cells.tsv:20 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T4 row 2（one-scenario） | cells.tsv:21 | 首稿把 build_pool、build_through_*、format_pool、MemoryPool::with_devices 四个具体构造函数名简化成泛指的 a for loop that builds a brand new pool | 补回四个具体名字：build_pool, build_through (加通配), format_pool, MemoryPool with_devices（英文里把下划线、双冒号按提示格式规则改写成空格，不是丢字） |
| T4 row 3（the whole gate） | cells.tsv:22 | 首稿把 oov-check.py、stage-mine.py、check-staged.sh、relabel-item.py、claim-experiment.sh 五个脚本名简化成泛指的 five specific scripts | 补回五个脚本名（去掉扩展名，只留主干名）：oov-check, stage-mine, check-staged, relabel-item, claim-experiment |
| T4 row 4（kb-shape） | cells.tsv:23 | 首稿把「1 未定项/已定项只许一种叫法；3 kb 文件不许链回自己；4 上游规则路径带 .claude/；5 …；7 …」里的编号 1/3/4/5/7（跳过 2、6，说明这一格只含其中几条子检查）简化成 Several sub checks bundled under one cell | 未逐一补回编号 1/3/4/5/7，因为这与 Q1 判据（落在哪一组、判据宽变窄）无关，只是子检查在原文件内部的编号；已在核对表这里写明缺了什么，供主 agent 判断是否需要更细 |
| T4 row 5（item-ref-status） | cells.tsv:24 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T4 row 6（status-redundancy） | cells.tsv:25 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T5 row 1（cross-decision-status） | cells.tsv:26 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T5 row 2（settled-item-self-open） | cells.tsv:27 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T5 row 3（settled-ref-says-open） | cells.tsv:28 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T5 row 4（stale-open-items） | cells.tsv:29 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T5 row 5（settled-same-file） | cells.tsv:30 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T5 row 6（freeze-layer-membership） | cells.tsv:31 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T6 row 1（decision-items-sync） | cells.tsv:32 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T6 row 2（blocking-verdict） | cells.tsv:33 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T6 row 3（user-verdict-owed） | cells.tsv:34 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T6 row 4（decision-summary-width） | cells.tsv:35 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T6 row 5（format-constants） | cells.tsv:36 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T6 row 6（clause-enums） | cells.tsv:37 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T7 row 1（feature-bits） | cells.tsv:38 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T7 row 2（entry-added） | cells.tsv:39 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T7 row 3（shape） | cells.tsv:40 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T7 row 4（status-sync） | cells.tsv:41 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T7 row 5（field-refs） | cells.tsv:42 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T7 row 6（field-projection） | cells.tsv:43 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T8 row 1（field-table-sums） | cells.tsv:44 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T8 row 2（first-txn-hooks） | cells.tsv:45 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T8 row 3（admission-terms） | cells.tsv:46 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T8 row 4（segment-registry） | cells.tsv:47 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T8 row 5（format-const-placeholders） | cells.tsv:48 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T8 row 6（second-txn-hooks） | cells.tsv:49 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T9 row 1（tree-table-reserve） | cells.tsv:50 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T9 row 2（mutation-tables） | cells.tsv:51 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T9 row 3（absolute-assertions） | cells.tsv:52 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T9 row 4（reading-discipline） | cells.tsv:53 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T9 row 5（index-sync） | cells.tsv:54 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T9 row 6（results-cited） | cells.tsv:55 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T10 row 1（evidence-in-repo） | cells.tsv:56 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T10 row 2（decision-links） | cells.tsv:57 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T10 row 3（verdict-false-named） | cells.tsv:58 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T10 row 4（repro-command） | cells.tsv:59 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T10 row 5（experiment-orphans） | cells.tsv:60 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T10 row 6（quoted-result-lines） | cells.tsv:61 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T11 row 1（archive-past-rounds） | cells.tsv:62 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T11 row 2（multipath-registry） | cells.tsv:63 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T11 row 3（table-shape） | cells.tsv:64 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T11 row 4（closeout-collects-open） | cells.tsv:65 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T11 row 5（paid-cited-tests） | cells.tsv:66 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T11 row 6（audit-contradictions） | cells.tsv:67 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T12 row 1（row27-preconditions） | cells.tsv:68 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T12 row 2（research-script-selftests） | cells.tsv:69 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T12 row 3（rules-manifest） | cells.tsv:70 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T12 row 4（stage-owners） | cells.tsv:71 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T12 row 5（agent-write-scope） | cells.tsv:72 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T12 row 6（change-range-single-source） | cells.tsv:73 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T13 row 1（research-gate-lint） | cells.tsv:74 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T13 row 2（fixture-claims） | cells.tsv:75 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T13 row 3（kb-registry） | cells.tsv:76 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T13 row 4（the whole gate） | cells.tsv:77 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T13 row 5（the whole gate） | cells.tsv:78 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T13 row 6（the whole gate） | cells.tsv:79 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T14 row 1（the whole gate） | cells.tsv:80 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T14 row 2（the whole gate） | cells.tsv:81 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T14 row 3（the whole gate） | cells.tsv:82 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T14 row 4（the whole gate） | cells.tsv:83 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T14 row 5（the whole gate） | cells.tsv:84 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |
| T14 row 6（the whole gate） | cells.tsv:85 | 无 | 逐句对齐 cells.tsv 第 4 列原文，未丢限定词、未加内容外的括注 |

## 二、背景与说明段落（Table A 参考表、格式规则、TB 三格）

| 提示里定位 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 背景段「A team wants to merge 23 existing gate scripts down to 15 scripts」及 Group 1 至 Group 10 整段 | _gate-shrink-r1-body.md:20-33（Table A 本身） | 无 | 逐组核对：目标名、旧文件清单、格数四项都与正文 Table A 逐行相符（Group 10「kept separate…」对应正文「各自一道」一行，54/55/57/59/87/77 六个文件名照抄） |
| 背景段「These ten groups' stated cell counts, added together, are 37 plus 10…」 | 首稿自拟框架句，非引文 | 首稿曾写成「The plan's own arithmetic states … which the plan claims equals 84」，把「84 是这十组格数之和」这句算术当成正文自己说过的话，而正文只写了「合计 15 道」（_gate-shrink-r1-body.md:35，指文件数不是格数）——这是首稿多出来、且来源站不住的一句归因 | 改写为中立陈述：只说十组格数相加是多少、cells.tsv 有 84 行，两者是否相符留给模型在 Q-COUNT 里自己算，不再把这句算术挂在「正文自己声称」名下 |
| 格式规则一至六（no markdown emphasis、编号答复、refuted-if 句、不写行号、无 shell、冒号后加空格） | 沿用 `research/prompts/gate-fix-forks-r3-local-attack.md:8-31` 同款六条格式规则 | 无 | 六条规则逐条对齐早先一轮本地攻方提示的写法，只把举例的编号从 T4-Q1、T8-Q1 换成这一轮自己的 T1-Q1、TB-Q1 |
| TB row 1（row27-preconditions）「what the inventory's dependency column says」一段 | cells.tsv:68 第 7 列（与谁重复：无（与 33 号 mutation-tables 的锚点计数同形不同事，本文件:9 gate-similar） | 把「33 号 mutation-tables」具体名字换成了泛指的 a different cell's anchor counting method | 保留「同形不同事」这个判断本身（这是核对表要核的实质），具体挂到哪个别的格因为与 TB-Q1 的判据（删掉后谁判）无关，未逐字补回名字；如需要可在核对表这里查到 |
| TB row 1「what the plan proposes」一段 | _gate-shrink-r1-body.md:41（B1 行）与 cells.tsv:68 第 8、10 列 | 无 | 「里程碑二提交之后删这一格」「今天写这一行已有去向」两处均与正文 B1 行及 cells.tsv 第 8 列逐项对齐 |
| TB row 2（agent-write-scope）整段「what the inventory's dependency column says」 | cells.tsv:72 第 7 列 | 首稿把「①②⑤⑥⑦⑨⑩⑫⑭⑯」十个具体编号简化成 ten specific numbered sub-checks，并把「四个钩子」简化成 four specific hooks，两处都没有点名具体是谁 | 补回：编号最高到 16（用「至少 16 条」这一可核推论明说是推出来的、不是原文直接给的总数）；四个钩子改用具体点名 write-guard, heavy-test-guard, ask-user-claim-guard, session-start（与 cells.tsv 原文「write-guard 没写 hook-events、heavy-test-guard 与 ask-user-claim-guard 只写 PreToolUse、session-start 只写 SessionStart」逐一对齐） |
| TB row 2「what the plan proposes」一段 | _gate-shrink-r1-body.md:42（B2 行） | 无 | 「删掉与共享 hooks-registered 重复的那一段，只留表与定义双向一致」与英文「delete the part…keeping only the write scope table consistency check, the definition header check, and the shared module check」逐项对齐（「shared module check」对应正文与 cells.tsv 里「两份共用模块的函数不另写一份」） |
| TB row 3（table-shape）整段「what the inventory's dependency column says」 | cells.tsv:64 第 7 列 | 无 | 「格数、已还行、表头三样 doc-lint 不判」逐项对齐英文「the cell count check, the paid off row shape check, and the header presence check are three things the shared linting stage does not judge」 |
| TB row 3「what the plan proposes」一段 | _gate-shrink-r1-body.md:43（B3 行） | 无 | 「删重叠的一半，留 doc-lint 不判的部分」与英文逐项对齐 |
