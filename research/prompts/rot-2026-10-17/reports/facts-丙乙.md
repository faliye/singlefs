# 事实表 丙乙（checks-owed 第三段、feature-bits、freeze-layer-membership、不变量清单、layout 01-03、milestone 01-02）

基准 ce1457c3，结束在工作区。没建副本、没起编译。

## 产出

- 事实表：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丙乙/facts-丙乙.tsv，40 行（F600-F639；`tail -n +2 facts-丙乙.tsv | wc -l` 得 40）。
- 候选表：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丙乙/candidates-丙乙.tsv，1610 行（`tail -n +2 candidates-丙乙.tsv | wc -l` 得 1610，不含表头；命令输出写 1347 个不同的行）。
- 各行候选数（`tail -n +2 candidates-丙乙.tsv | cut -f1 | sort | uniq -c`）：F600 152、F601 3、F602 14、F603 9、F604 15、F605 31、F606 23、F607 45、F608 25、F609 184、F610 228、F611 41、F612 270、F613 43、F614 7、F615 3、F617 61、F618 6、F619 4、F620 2、F621 5、F622 7、F623 23、F624 49、F625 40、F626 21、F627 32、F628 7、F629 26、F630 15、F631 71、F632 3、F633 68、F634 52、F635 25。F616（新立）与 F636-F639（只改措辞）不出候选。

## 两条命令

`python3 scripts/kb/stale-candidates.py --check-facts .../facts-丙乙.tsv --base ce1457c3`，退出码 6（预期：67 条 H 编号不在我片里）。输出里「事实表没罩全」那一行（原样）：

    ✗ 事实表没罩全：67 条变更记录没罩、0 行正则坏了、0 行检索词在基准零命中、0 行在结束零命中、0 行检索词太宽、0 行多余的 &&、0 行冒充新立、0 行冒充只改措辞、0 行宽词没抽查

它后面还有一行「→ 怎么办：…」的提示，不是结论。除「67 条没罩」外各项红都是 0。

`python3 scripts/kb/stale-candidates.py --facts .../facts-丙乙.tsv --base ce1457c3 --out .../candidates-丙乙.tsv`，退出码 0。输出末行（原样）：

    ✓ 候选表 /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丙乙/candidates-丙乙.tsv：1610 行（1347 个不同的行）

## 事实表各行一览

| 编号 | 事实 | 检索词类型 |
|---|---|---|
| F600 | 不变量清单搬家：`.claude/kb/invariants.md` 一份 -> `crates/singlefs-checker/invariants/` README + 九份，数法 = 92 | 旧路径 `invariants\.md` |
| F601 | 条数：kb 写 90 条已实现/113 条在用 -> 92/115，清单收进 I-9.36、I-9.37 | 旧数字串 |
| F602 | 清单状态节：新池新建文件阶段 -> 当前里程碑「删除与多次 COW」 | `当前里程碑` |
| F603-F608 | 清单逐条内容差异（我 diff 过：把两边的日期、层 0 规范化后，状态列/判据列真不同的行）：I-7.4（C556 欠 -> 已还清）、I-8.3（C37 与崩溃点重放那一半欠 -> 已做）、I-3.8（行只增不删 -> 清扫删行）、I-8.10（新根段四段 -> 七段）、I-8.5（测试改名）、I-9.14（镜像改名，欠账已改） | 条目编号 |
| F609 | 欠账表每笔带「去向：里程碑三…」（旧 0 处 -> 173 行） | `^\| C[0-9]+ \|.*去向` |
| F610 | 24 笔挪进已还清（2026-10-11；C760、C771 并进 C768、C682） | C 编号串 |
| F611 | C613、C629、C644、C646 挪进已还清（2026-10-13、2026-10-15） | C 编号串 |
| F612 | 25 笔整笔还欠 -> 半还，只剩还欠的一半 | C 编号串 |
| F613 | 8 笔前置「条款没定/交主 agent 或用户定」-> 2026-10-12 判决、条款已定 | C 编号串 |
| F614 | C589、C590 前提不在（变更史现状行同步格已删） | C 编号 + status-sync |
| F615 | 14 笔简称改名（旧名作检索词，所以候选少） | 旧简称 |
| F616 | 新立：C774-C801、C830-C846 共 45 笔（基准零命中，逐个 `git grep` 核过） | C 编号区间 |
| F617 | 全绿标记 -> 崩溃判定库（C564） | `全绿标记` |
| F618 | FULL_EXPAND_UP_TO 28 -> 30 | 常量名 |
| F619 | GPU 内核展开大小从只钉上界改成钉具体数（C722） | 数字串 |
| F620 | 「26.16 亿态零违例」多记录发布一项 exhaustive=false（C761） | 旧说法 |
| F621 | status-sync 格删，改 index-sync 格 | 格名 |
| F622 | 谱系记录宽 16 -> 24（H_cur）；退役记录容器的容器索引 = 退役树 | 字段串 |
| F623 | feature-bits 位 1 的引进提交：待提交 -> 45d49aaf | 常量名 |
| F624 | 树表 11 -> 12 条、树 ID 水位 65548 -> 65549、已承诺预留 98 386 -> 196 760 | 数字串 |
| F625 | 新池新建文件流全量闭式 262165 -> 16 777 240；覆盖写固定脚本流 481 次写、1 678 377 204 态 | 数字串 |
| F626 | E142 主产物 r21 -> r24（r25 量出逐字相同）；mkfs 行 12+1+1+1+4 与用例 14+1+1+1+4 差清零一步 | 产物名 |
| F627-F635 | layout/milestone 的实质改写：回退到无文件那一版成非候选、抬 F 正常触发已做、deleted_inodes 形态已定、系统配置读法改序、nonce/MAC 行改写、layout 02 登记位已定、layout 03 干跑已核、回退行退役、隔离位清零与 C493 | 概念名 |
| F636-F639 | 只改措辞：层 0 -> 崩溃点重放（及 layer0 改名、第二条流 -> 覆盖写固定脚本流）、行号 -> 锚串、上游 test-script -> 路径、今天 -> 删/当时 | 不出候选 |

## 判为「只改措辞」的载体（我片里）

只改措辞的内容，对应事实表 F636-F639（旧事实写「只改措辞」，不出候选）：

- `.claude/kb/checks-owed.md`（第三段）：行号引用（「第 N 行」「文件:行号」）换成函数名、小节名、变异名锚串；「上游 `test-script-discipline.md`」写成 `.claude/rules/test-script-discipline.md`；今天 -> 现行/删掉；层 0 -> 崩溃点重放；`checker-tier-layer0-replay` -> `checker-tier-crash-point-replay`；`.claude/kb/invariants.md` -> `crates/singlefs-checker/invariants/`（路径搬迁部分）；C 编号简称的补括注。
- `.claude/kb/layout/01-first-txn.md`：行号 -> 锚串、「2026-09-xx 起」删日期、「第二条流」-> 覆盖写固定脚本流、层 0 -> 崩溃点重放、门禁名改名、「重量归提交时的崩溃验证员」-> 主 agent。
- `.claude/kb/layout/02-second-txn.md`：层 0 -> 崩溃点重放、「五种盘上形态」去「五种」、「10 条单测」去数。
- `.claude/kb/milestone/01-first-txn.md`：层 0 -> 崩溃点重放（步 7 标题、门禁名）、层 1 -> 常规层/层 0 全量 -> 冒烟层全量（D13 三层的叫法）、删「2026-09-14 起」、「今天」-> 当天。
- `.claude/kb/milestone/02-second-txn.md`：层 0 -> 崩溃点重放、门禁名改名、`enumerate_layer0_versions` -> `enumerate_crash_states_versions`、`LAYER0B` -> `CRASH_POINT_REPLAY_FIXED_SCRIPT`、第一/二条流改称、今天 -> 当时/2026-09-16 时（日期化的部分），「状态：2026-09-29 判定结束」删日期、此前 -> 那之前。
- `.claude/kb/freeze-layer-membership.md`、`.claude/kb/feature-bits.md`：只有「今天没有」-> 「没有」、「现在什么状态」-> 「什么状态」这两处是措辞；其余（谱系记录宽、容器索引、位 1 的引进提交）是事实，已在 F622、F623。
- `crates/singlefs-checker/invariants/*.md`：状态列去「2026-09-xx，」日期前缀、层 0 -> 崩溃点重放、`walk.rs:1220` 一类行号 -> 函数名、「invariants.md」-> 「不变量清单」、N -> n 的大小写；与 kb 旧文件逐条规范化（去日期、层 0 换名）后仍不同的行都进了 F603-F608 和 F601。

## 命中超过 100 行的检索词：抽查

抽法：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丙乙/samp.py`（同一份现状载体集合，随机种子 1-4，抽 10 行）。

| 编号 | 命中行数 | 抽 / 相干 | 说明 |
|---|---|---|---|
| F600 `invariants\.md` | 152 | 10 / 10 | 10 行全是提到旧路径的句子，多数在 records 里（事件句，judge 时多半判「事件句不改」） |
| F609 `^\| C[0-9]+ \|.*去向` | 184 | 10 / 10 | 10 行全是 checks-owed 里带去向的欠账行本身；它们是新写法，不会是腐烂，候选基本判「不相干」。这一行收益低，主 agent 可以决定整行删掉 |
| F610（24 笔 C 编号） | 228 | 10 / 9 | 1 行只是顺带提到 C604-C608 区间 |
| F612（25 笔 C 编号） | 270 | 10 / 9 | 1 行（「C715、C716 插在 C714 之后」）顺带 |

其余检索词都在 100 行以内（`--check-facts` 报「0 行宽词没抽查」）。

## 要主 agent 现查（不在事实表形态里的可疑之处）

1. **`stale-candidates.py` 的现状载体不含不变量清单的新家。** `scripts/kb/stale-candidates.py` 第 65 行起 `CARRIER_PATTERNS` 只列 CLAUDE.md、README.md、`.claude/` 下的 agent-common、main-agent、agents、rules、skills、handover、kb、`records/*.md`；`crates/singlefs-checker/invariants/*.md`（README + 九份，搬家后清单的唯一住处）不在里面，所以 F601-F608 的候选搜不到它们自己，清单里的现状句没有被机械扫到。要加 `^crates/singlefs-checker/invariants/.+\.md$`，或主 agent 另派人读这 10 份。
2. **已承诺预留的两个数打架。** `.claude/kb/milestone/02-second-txn.md` 第 128 行写「已承诺预留仍是第一个文件版本那次的 98 386」；`.claude/kb/layout/01-first-txn.md` 第 304 行与 `.claude/kb/layout/03-third-txn.md` 第 22 行把同一个量（新池新建文件那一版记账条目的已承诺预留）改成 196 760（式子多了退役树）。milestone/02 那一句可能该跟着改（F624 会列出，judge 时核）。
3. **已还清表里 8 笔「怎么还的」指向不存在的节。** `.claude/kb/checks-owed.md` 第 735-742 行：C626、C627、C601、C607、C608、C620、C674、C676 的第三列写「见 C626（…） 节」「见「拟写的句子」节 C601」，但这份文件只有 `# 欠的检查清单` 与 `### 已还清` 两个标题（`grep -n '^#' .claude/kb/checks-owed.md` 得 1 与 576 两行），没有这些节；基准那一版里这种写法 0 处（`git show ce1457c3:.claude/kb/checks-owed.md | grep -c '^| C[0-9]* | [^|]* | 见 C'` 得 0）。这 8 笔是怎么还的，表里没写，要补一句或指到真有的位置。
4. **基准那一版的不变量清单本来就落后代码。** `git show ce1457c3:crates/singlefs-checker/src/image.rs` 里 `IMPLEMENTED_INVARIANTS: [&str; 92]` 已是 92，kb 清单当时写 90 条/113 条在用（缺 I-9.36、I-9.37）。所以「90 -> 92」不是这一阶段的代码变化，是搬家时补齐了文档。F601 的旧事实照基准那一版的 kb 写。
5. 仍写 `invariants.md`（旧单文件名）的现状载体，非 records：`.claude/kb/experiments/170-O3规约执行器第一版.md` 第 121、338 行，`171-三oracle合谋测试.md` 第 106、172、207 行，`94-搬共享落点的触碰面.md` 第 58 行，`.claude/kb/prior-art.md` 第 169 行；这几处的命名路径 F600 会列出，judge 时判是历史叙述还是现状。
6. checks-owed 第三段里我没逐条读出语义的行：分到我片里的变更是字符窗口式的碎片（单个碎片只有几十字），我按 C 编号把「新旧两版的整行」对出来分类（脚本 `cols.py`，对 old/new 两份 checks-owed 按 `| C… |` 行比），不是靠碎片本身。已分类：挪进已还清 28 笔、半还 25 笔、前置改写 31 笔、简称改名 14 笔；其余主要是去向附加、去日期、行号换锚、列内改写（我片里共 339 行，其中 85 行两版逐字相同，没逐笔读语义）。分类按关键词和列差异做的，半还/前置改写的边界笔可能漏或错归，judge 时看到「前置」一格实质变了的行请照实判。

## 没做什么

- 没读 `research/prompts/` 与 `briefs/`，没碰 `~/.claude/` 会话记录。
- 没建副本、没起编译、没跑门禁。
- 只写事实表，不判候选；`--check-facts` 里的 67 条 H 编号没罩住是预期的（不在我片里）。
- invariants 九份类文件我只做了「旧单文件 vs 新九份」的逐行规范化 diff（脚本比对条目列、简称列、状态列），没有逐字读完全部 121 条的判据正文。
- 草稿目录里留了 `old-owed.md`、`new-owed.md`、`old-invariants.md`、`new-invariants.md` 几份对照文本与脚本，不是仓副本，没有编译目录。
