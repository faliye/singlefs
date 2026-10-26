# 阶段同步第二段：逐行判 F67-F72（batch-2026-10-21）

候选表：/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv，sha256 a2913e391a222ec455f068bb72d7123c94659be2da960181125ecf2bc132e6d9（与派发提示的 a2913e39… 相符）。
分到的组 F67-F72 在候选表里实有 F67、F68、F69、F70、F72 五组（没有 F71，它是新立行、不出候选），共 221 行，每行一条判定；候选表的原文列只有前 500 个字，超出部分我回到仓里原行去读。
只判不改，没改任何仓内文件；草稿放 scratchpad，没有建编译目录或仓副本。

## 结论一览

- 要改 17 行（表内）：F72 的 INDEX.md:28 一行，kb 里里程碑 02/03/04 的 16 行 `briefs/….zh.md` 路径引用（简报已在 1bb3a24dc 改名为 `.md`，现无任何 `.zh.md`）。
- 要补 3 行：欠账 C602 的三语三行，没记 2026-10-21 又积压 471 份、已归档。
- 候选之外我顺手查到 3 条要改，列在表尾 M1-M3：INDEX.en.md:28、INDEX.ja.md:28 与 `lib-governance-refs.py` 一族还按旧约定扫 `README.zh.md`。
- F67（clock-times 认挂载设备号）、F68（abandoned-rounds 只读顶层、补回 m3-batch63-code-r1）、F70（撤两张已废红例账登记表）三组：没有一行现状句在说旧的，全是不相干或事件句。
- 事实是否进了该记的载体（第 10 步反向问）：F67、F68、F69 在 `records/2026-10-21-doc门禁违规修复.md` 有记；F70 在 `records/2026-10-18-审查报告核实与修复.md` 与 `.claude/skills/crash-test/commit-stages.md`、`.claude/rules/verification.md` 有记；F72 在 `records/2026-10-19-kb三语门禁与skill.md`、`CLAUDE.md` 第 101 行、`.claude/kb/translation-glossary.md` 第 3 行、`.claude/skills/trilingual/SKILL.md` 有记。没有再加 M 行去补记。

## 现查依据（原样输出）

```
$ awk -F"\t" '$1 ~ /^F(67|68|69|70|72)$/' candidates.tsv | wc -l
221
$ for g in F67 F68 F69 F70 F72; do echo $g $(awk -F"\t" -v g=$g '$1==g' candidates.tsv | wc -l); done
F67 26
F68 40
F69 63
F70 53
F72 39
$ sha256sum candidates.tsv
a2913e391a222ec455f068bb72d7123c94659be2da960181125ecf2bc132e6d9  /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv
$ ls briefs
2026-09-14.en.md
2026-09-14.ja.md
2026-09-14.md
2026-09-28.en.md
2026-09-28.ja.md
2026-09-28.md
2026-10-10.en.md
2026-10-10.ja.md
2026-10-10.md
2026-10-18.en.md
2026-10-18.ja.md
2026-10-18.md
$ find briefs -name "*.zh.md" | wc -l
0
$ grep -n "^### 全量" briefs/2026-09-28.md; grep -n "^### 这个结果不能读成" briefs/2026-10-10.md
74:### 全量（2026-09-30）：七个分项 26.16 亿态零红，7 小时 37 分
89:### 这个结果不能读成「验过了、没有红」
$ grep -rn "briefs/[0-9-]*\.zh\.md" --include=*.md . | grep -v "^./research/prompts/\|^./records/" | wc -l
33
$ sed -n 101p CLAUDE.md | grep -o "YYYY-MM-DD.md. 是中文（默认）"
YYYY-MM-DD.md` 是中文（默认）
$ cut -f1,2 research/prompts/abandoned-rounds.tsv | grep -v "^#"
c196-r1	2026-09-12
d16-item1-r2	2026-09-11
c364-r3	2026-09-16
smalldata-ttime-r1	2026-09-20
m3-batch63-code-r1	2026-10-11
$ grep -c "归档\|archive" .claude/skills/crash-test/commit-stages.md .claude/skills/gate/staged-gate.md
.claude/skills/gate/staged-gate.md:0
.claude/skills/crash-test/commit-stages.md:0
$ ls .claude/gate.d/e165-void-red-example-ledgers.tsv .claude/gate.d/e165-superseded-red-example-ledgers.tsv
ls: cannot access '.claude/gate.d/e165-void-red-example-ledgers.tsv': No such file or directory
ls: cannot access '.claude/gate.d/e165-superseded-red-example-ledgers.tsv': No such file or directory
$ grep -c "e165-void\|e165-superseded" .claude/skills/crash-test/commit-stages.md
0
$ sed -n 28p .claude/kb/INDEX.md .claude/kb/INDEX.en.md .claude/kb/INDEX.ja.md
| [translation-glossary.md](translation-glossary.md) | 翻译术语表：中、英、日三语并排，仓根 README 与 `briefs/` 的三语版本照它用词 |
| [translation-glossary.md](translation-glossary.md) | Translation glossary: Chinese, English, and Japanese side-by-side; trilingual versions of the root README and `briefs/` follow its terms |
| [translation-glossary.md](translation-glossary.md) | 翻訳用語集: 中国語・英語・日本語の三言語を並記、ルート README および `briefs/` の三言語版はその用語に従う |
```

## 看点说明

- 判「事件句不改」的，是 records 里按时间写的经过、交回、当日读数与派活：把旧说法换成新的，这句还是不是真话——换了就不再是那一天发生的事，所以不改。
- 判「不相干」的，是提到了检索词、说的却不是这件事实的现状句，每条理由引了原话并写了为什么不受影响。F70 的 `rules/verification.md:126`、F72 的 `translation-glossary.md:3`、`SKILL.md:3`、`CLAUDE.md:102` 本来就是新说法，没有旧说法要改，也归「不相干」。
- C602 判要补而不是要改：它写「提交流程里单列一步还没有」，现查 commit-stages.md、staged-gate.md 里归档 0 处，这句仍真；缺的是新一次积压。
- 候选表里 `.claude/kb/milestone/04-fourth-txn.md` 是第 74 行，我现查时 `grep` 给的是第 75 行（别的会话在改同一份文件，行号随之移了一行），表里按候选表的行号写。

## 逐行判定

### F67

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F67 | .claude/agent-common.md:26 | 不相干 | 规则句只规定报告里时间写日期，并点出 clock-times 格与写闸在判，没有说挂载设备号算不算钟点，不受这件事影响；引原话「oc-text 的 clock-times 格判，Write / Ed」 |
| F67 | .claude/kb/checks-owed.en.md:556 | 不相干 | C838 是文本页带 NUL 让按改动判的格整份跳过的欠账，clock-times 只是被举例的一格，设备号认法不动它的任何分句；引原话「oc-text 的 clock-times, private-valu」 |
| F67 | .claude/kb/checks-owed.ja.md:556 | 不相干 | 译文同上：C838 是 NUL 让按改动判的格整份跳过，与设备号认法无关；引原话「oc-text 的 clock-times、private-value」 |
| F67 | .claude/kb/checks-owed.md:551 | 不相干 | C838 是 NUL 让按改动判的格整份跳过的欠账，与设备号认法无关；引原话「oc-text 的 clock-times、private-value」 |
| F67 | records/2026-09-16-subagent拆分提案.md:977 | 不相干 | 09-16 提案的现象栏，讲当年判据只认时:分、漏了 N 点写法，是那一次的缘起，没提设备号；引原话「ate.d/lib-clock-times.py` 只认「时:分」。用」 |
| F67 | records/2026-09-16-subagent拆分提案.md:978 | 不相干 | 同一记录的改法栏，给 lib-clock-times 加四个钟点形态，说的是当年的形态，没提设备号；引原话「改法   `lib-clock-times.py` 加四个形态：日期或」 |
| F67 | records/2026-09-16-subagent拆分提案.md:980 | 不相干 | 同日续里加 bare-clock 等形态与全仓改写日期，是那一轮的做法，没提设备号；引原话「ons.py`），`clock-times` 形态加 `bare-cl」 |
| F67 | records/2026-09-28-agent体系用量时间与调度分析.md:345 | 不相干 | 09-28 记录里 clock-times 格当日红 84 行的读数，设备号不在其内；引原话「oc-text 的 clock-times 格   84 行红，只有」 |
| F67 | records/2026-09-28-agent体系用量时间与调度分析.md:356 | 不相干 | 同记录里冻结证据的钟点登记进 excluded_prefixes 的处理，是当日做法，与设备号无关；引原话「oc-text 的 clock-times 格：冻结证据里 diff」 |
| F67 | records/2026-09-28-门禁59号提速与双机分片.md:33 | 不相干 | 59 号提速记录里把 12 号与 17 号合成一道的分工，与设备号无关；引原话「号与样本、`lib-clock-times.py`、`.claude/」 |
| F67 | records/2026-10-11-scripts审计-第二轮.md:23 | 不相干 | scripts 审计的一条发现：apply-writer-patch 缺 clock-times:allow 标记，与设备号无关；引原话「」）；这一行没有 `clock-times:allow` 标记，`me」 |
| F67 | records/2026-10-12-里程碑三工具与治理批.md:158 | 不相干 | 第二题攻方腿的交回清单，clock-times 只作为「截在私有值中间」的例子出现，与设备号无关；引原话「names 红行与 clock-times 截在私有值中间，改前也中）」 |
| F67 | records/2026-10-12-里程碑三工具与治理批.md:164 | 不相干 | 集成真树上 clock-times、term-renames 两格红的当日分诊，与设备号无关；引原话「r` 在集成真树上 clock-times、term-renames」 |
| F67 | records/2026-10-13-复用键与全仓改名.md:378 | 不相干 | 合入后验证记录，clock-times 119 行是当时的数，设备号不在其内；引原话「- 合入后验证：`cargo check --workspa」 |
| F67 | records/2026-10-13-欠账还五条.md:355 | 不相干 | 钟点检查判腿产物 4 行红、改冻结证据的当日处理，与设备号无关；引原话「oc-text 的 clock-times 格）判 `c439-r1`」 |
| F67 | records/2026-10-13-欠账还五条.md:356 | 不相干 | 同记录里给固定提交时刻加 clock-times:allow 的处理，与设备号无关；引原话「提交时刻加了 `# clock-times:allow`（改前 sha」 |
| F67 | records/2026-10-13-里程碑四预开发.md:432 | 不相干 | 里程碑四预开发把 clock-times 等红点名归阶段同步，是当时的分流，没提设备号；引原话「ef-status、clock-times、session-names」 |
| F67 | records/2026-10-18-审查报告核实与修复.md:25 | 不相干 | 审查报告核实表里 clock-times 5 行的当日数，与设备号无关；引原话「四格   成立   clock-times 5 行、session-n」 |
| F67 | records/2026-10-18-里程碑三收尾与近期欠账.md:276 | 不相干 | 里程碑三收尾记录里 clock-times 只是「doc-text 已有同类格」的落点理由，与设备号无关；引原话「- 用户 2026-10-19：「18 条找不到归属的欠账放」 |
| F67 | records/2026-10-19-文档普查报告核实与修复.md:37 | 不相干 | 文档普查记录里「clock-times 12 行在别的会话的文件里」是当日分诊，设备号不在其内；引原话「在用」实际 123；clock-times 12 行在别的会话的文件里」 |
| F67 | records/2026-10-21-doc门禁违规修复.md:65 | 事件句不改 | 这一轮 D 包「没改」那句：e185 第 9047 行是 mountinfo 的主次设备号、同一行加不了 clock-times:allow、交下一批工具实现员；是那一次的处置，后来批次 1 丙让格认设备号；引原话「- D 包主 agent 自己改（没人认领的几份，repla」 |
| F67 | records/2026-10-21-doc门禁违规修复.md:82 | 事件句不改 | 开工现查的读数：clock-times 38 行里这一轮管的只有 e185 第 9047 行；那一天的数；引原话「c`，逐格点名跑）：clock-times 38 行，其中这一轮管的只」 |
| F67 | records/2026-10-21-doc门禁违规修复.md:88 | 事件句不改 | 依赖表里派给丙的任务行；是这一轮派活时的计划，做成见同记录丙交回那一条；引原话「（简称）` 节标题；clock-times 认 mountinfo 的」 |
| F67 | records/2026-10-21-doc门禁违规修复.md:97 | 事件句不改 | 批次 1 派出的记录，列丙的活，当时派出了什么；引原话「孤立译文不顶替原文、clock-times 认 mountinfo 设」 |
| F67 | records/2026-10-21-doc门禁违规修复.md:105 | 事件句不改 | 丙交回：clock-times 认 mountinfo 设备号并记开关 LIB_FORBIDDEN_NOTATIONS_BREAK=mountinfo-device-numbers；那一次交回的内容；引原话「bstitute`；clock-times 认 mountinfo 设」 |
| F67 | records/2026-10-21-doc门禁违规修复.md:106 | 事件句不改 | 暂存后真仓单跑：clock-times 从 38 行到 37 行、不再点名 e185；那一次跑的读数；引原话「1 个 blob）；clock-times 从 38 行到 37 行，」 |

### F68

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F68 | .claude/agent-common.md:87 | 不相干 | 派发闸一节说 abandoned-rounds 格与 ⑥ 共用一份判法（日期、同一轮重复、出处、旧行名单），与归档历史读不读子目录无关；四行旧登记与 tsv 现有的 c196-r1、d16-item1-r2、c364-r3、smalldata-ttime-r1 四行一致；引原话「- 派发闸（`.claude/hooks/runner-di」 |
| F68 | .claude/kb/checks-owed.en.md:268 | 不相干 | C364 一行说第三轮登记在 abandoned-rounds.tsv；现查该表仍有 c364-r3 行，句子仍真，也没说归档历史怎么读；引原话「C364   整理意图记录没有字段表   D26 (Ba」 |
| F68 | .claude/kb/checks-owed.ja.md:268 | 不相干 | 译文同上：第三轮登记在 abandoned-rounds.tsv，表里仍有 c364-r3；引原话「C364   整理意图记录没有字段表   D26（バック」 |
| F68 | .claude/kb/checks-owed.md:267 | 不相干 | C364 一行说第三轮登记在 abandoned-rounds.tsv，表里仍有 c364-r3，句子仍真；引原话「C364   整理意图记录没有字段表   D26（后台整」 |
| F68 | .claude/kb/milestone/02-second-txn.en.md:359 | 不相干 | 收口表第 33 行写门禁 abandoned-rounds 格已立，是设立的事实，没说它读归档历史的范围；引原话「ords cell abandoned-rounds established (」 |
| F68 | .claude/kb/milestone/02-second-txn.ja.md:359 | 不相干 | 译文同上：abandoned-rounds 格已立，不涉及读归档历史的范围；引原话「records の abandoned-rounds セルを設置（.claude」 |
| F68 | .claude/kb/milestone/02-second-txn.md:373 | 不相干 | 收口表第 33 行写 abandoned-rounds 格已立，不涉及读归档历史的范围；引原话「records 的 abandoned-rounds 格已立（`.claude/」 |
| F68 | .claude/main-agent.md:23 | 不相干 | 主 agent 入口说撂下的轮要在 tsv 登记一行才不再挡，讲的是登记写法，与读归档历史的范围和 m3-batch63-code-r1 补回无关；引原话「4. **派实现员之前先聚簇，不为一条发现单派**：把这一轮」 |
| F68 | .claude/rules/three-way-inference.md:121 | 不相干 | 规则讲撂下轮的登记行写法，与读归档历史的范围无关；引原话「h/prompts/abandoned-rounds.tsv` 登记一行收开工快」 |
| F68 | records/2026-09-16-subagent拆分提案.md:548 | 事件句不改 | 09-16 提案里这个阶段设立时的设计（发过腿而没有判决即红），是当时的方案；引原话「gate.d/66-abandoned-rounds.sh`（归 `gate-t」 |
| F68 | records/2026-09-19-里程碑二遗留收拢.md:30 | 不相干 | 09-19 任务表里「两轮撂下的三方照 tsv 重出材料」，说的是别的两轮，与本阶段的门禁修法和 m3-batch63 行无关；引原话「h/prompts/abandoned-rounds.tsv` 重出材料」 |
| F68 | records/2026-09-28-里程碑二收口表回写与系统配置槽写频率门禁.md:36 | 事件句不改 | 09-28 记录里「11 号 abandoned-rounds 红的三轮是别的会话在跑的」，当日读数；引原话「h/prompts/abandoned-rounds.tsv` 的 c245-r」 |
| F68 | records/2026-09-28-里程碑二收口表回写与系统配置槽写频率门禁.md:37 | 事件句不改 | 判决写完时删 c245-r3 行的计划，那一次的安排；引原话「h/prompts/abandoned-rounds.tsv` 的 c245-r」 |
| F68 | records/2026-09-28-里程碑二收口表回写与系统配置槽写频率门禁.md:38 | 事件句不改 | 当日审查改动清单点名 abandoned-rounds.tsv，是那一次的事；引原话「表 C245 行，`abandoned-rounds.tsv`，本记录）；E1、」 |
| F68 | records/2026-09-28-里程碑二收口表回写与系统配置槽写频率门禁.md:41 | 事件句不改 | c245-r3 判决写完后该行已删，那一次的事；引原话「3（改法丁零轮）；`abandoned-rounds.tsv` 的 c245-r」 |
| F68 | records/2026-10-01-删除与并发事务.md:1602 | 事件句不改 | 10-01 记录的两处欠：当时派发闸不读该表；后来 ⑥ 已改读，是那一天的状况；引原话「h/prompts/abandoned-rounds.tsv`（撂下的轮照旧挡派」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:73 | 事件句不改 | 10-12 批里写「tsv 已有 m3-batch63-code-r1 一行」，是那一天的读数；引原话「h/prompts/abandoned-rounds.tsv` 已有 m3-ba」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:104 | 事件句不改 | 攻方腿打中的清单项，其中 halted 与 tsv 两处登记是那一轮的发现；引原话「halted 与 `abandoned-rounds.tsv` 两处登记；D7「」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:108 | 事件句不改 | 批 4 的分工派发，G10 ⑥ 改读 tsv，那一次的计划；引原话「G10 ⑥ 改读 `abandoned-rounds.tsv`、撤 halted」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:109 | 事件句不改 | 批 4 丁交回的待处置，列 m3-batch63 出处缺行号，那一次的交回；引原话「类型表、⑥ 指到 `abandoned-rounds.tsv`）。待处置：① `」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:112 | 事件句不改 | 批 4 甲交回，给 m3-batch63 行补第 1595 行，那一次的交回；引原话「halted、读 `abandoned-rounds.tsv`（出处 `reco」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:115 | 事件句不改 | 复核脚本分诊：c196-r1 等三轮的腿文件已归档，集成副本的假象，那一次的事；引原话「；② 第 16 项 abandoned-rounds 格红在 c196-r1、c」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:141 | 事件句不改 | 批 8 的分工，列样本与库，那一次的计划；引原话「- 新题第二轮辩方腿交回（`m3-defs-b6-r2-so」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:153 | 事件句不改 | verify12 分诊，三轮材料在 3cff909b 归档时删了，那一次的事；引原话「es 一行、199 abandoned-rounds 认不出 c196-r1、c」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:157 | 事件句不改 | 冻结副本拷贝里跑样本档 22 例判对，那一次的结果；引原话「22 例判对（含 abandoned-rounds-source-green」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:177 | 事件句不改 | 更正记录，说批 11 样本里的旧日期行，是那一次的更正；引原话「批 11 改过的 `abandoned-rounds-source-green`」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:181 | 事件句不改 | 批 12 交回，新开关与样本，那一次的交回；引原话「pt`、新绿样本 `abandoned-rounds-source-date-p」 |
| F68 | records/2026-10-12-里程碑三工具与治理批.md:183 | 事件句不改 | 合并核对的记录，tsv 出处另补是当日冲突决议之一；引原话「登记值后删豁免行、`abandoned-rounds.tsv` 出处另补、hoo」 |
| F68 | records/2026-10-13-代码欠账批.md:169 | 事件句不改 | 代码欠账批里 abandoned-rounds 格红在 d26-item4-r1，当日处理；引原话「records 的 abandoned-rounds 格红在 d26-item4」 |
| F68 | records/2026-10-13-复用键与全仓改名.md:37 | 事件句不改 | 当日删 tsv 里认不出材料的三行（9 行到 6 行）的处置；后来 04ceaa904 归档又删了 m3-batch63 行，本阶段取回；引原话「records 的 abandoned-rounds 格报 `research/」 |
| F68 | records/2026-10-13-复用键与全仓改名.md:52 | 事件句不改 | 当日把 tsv 文件头「66 号」改成格名的一次改动；引原话「- 2026-10-13：文档收尾 clerk（报告 `~/」 |
| F68 | records/2026-10-13-复用键与全仓改名.md:378 | 事件句不改 | 合入后验证里 abandoned-rounds 红是提交前清单的治理欠账，当日分诊；引原话「- 合入后验证：`cargo check --workspa」 |
| F68 | records/2026-10-13-复用键与全仓改名.md:387 | 事件句不改 | 当日补 m3-batch63 出处并从 HEAD 找回三行的处置，那一次的事；引原话「一行并写明补写日期；abandoned-rounds：工作区的 `abandon」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:48 | 事件句不改 | 第 1 批重派的派活，列盘点 abandoned-rounds 三轮的来历，那一次的派发；引原话「过程类几格要做什么、abandoned-rounds 三轮的来历，交分包建议（草」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:50 | 事件句不改 | 主 agent 现查三条，发现门禁读 git log 取 basename 不限顶层是 bug，是那一次的发现，修在同记录第二位交回那一条；引原话「:1595` 还在；abandoned-rounds 读 `git log --」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:53 | 事件句不改 | 主 agent 把 m3-batch63 撂下登记从 04ceaa904^ 原样取回，那一次的操作；引原话「h/prompts/abandoned-rounds.tsv`（`04ceaa9」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:54 | 事件句不改 | 第 3 批第一波派活，把 abandoned-rounds 门禁 bug 排下一波，那一次的计划；引原话「records 的 abandoned-rounds 门禁 bug 排下一波；别」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:61 | 事件句不改 | 派第二位 tooling-writer 的记录，那一次的派发；引原话「量表行数进实际条数；abandoned-rounds 读归档历史不限顶层（草稿」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:69 | 事件句不改 | 进度快照：按时间写进「## 经过」的那一刻「还开着」，随后一条就是第二位的交回；引原话「ape、不变量计数、abandoned-rounds）；item-ref-sta」 |
| F68 | records/2026-10-21-doc门禁违规修复.md:71 | 事件句不改 | 第二位交回：abandoned-rounds 读归档历史只认顶层，就是这件事实本身，那一次的交回；引原话「际条数只数中文原文；abandoned-rounds 读归档历史只认 `rese」 |

### F69

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F69 | .claude/agent-common.md:30 | 不相干 | 规则句：早于本轮的实验记录不留在工作区、下一次提交删上一批；471 份归档不改这条规则，仍真；引原话「riments 的 archive-past-rounds 格判这一条）。」 |
| F69 | .claude/kb/checks-owed.en.md:311 | 不相干 | C436 讲归档删掉的路径被反引号引用时两道都够不着，是脚本扫描范围的缺口；471 份归档不改它；引原话「三层各自够不着: `archive-past-rounds.py` 的 `still_」 |
| F69 | .claude/kb/checks-owed.en.md:350 | 不相干 | C449 里提 archive-past-rounds.py 的 base_of 说明写错变量名，与归档了几份无关；引原话「cripts/kb/archive-past-rounds.py` 的 base_of」 |
| F69 | .claude/kb/checks-owed.en.md:392 | 要补 | C602 的「积压那一批已由提交 31128273 归档」之后补一句：2026-10-21 又积压 471 份（里程碑二、三与更早实验的记录），照 GATE_BASE=1bb3a24dc 的 --apply --keep-registered 归档（records/2026-10-21-doc门禁违规修复.md）；这笔欠账 提交流程里单列一步 仍没有（commit-stages.md 与 staged-gate.md grep 归档 0 处），英语译文同补：On 2026-10-21 another 471 files had accumulated (records of Milestone 2, 3 and earlier experiments) and were archived with --apply --keep-registered under GATE_BASE=1bb3a24dc；原话「riments 的 archive-past-rounds 格只判"工作区里不许留上一」 |
| F69 | .claude/kb/checks-owed.en.md:814 | 不相干 | C437 讲归档遇 gitlink 异常中止，2026-10-13 已修，与本阶段无关；引原话「cripts/kb/archive-past-rounds.py` 的 past_ro」 |
| F69 | .claude/kb/checks-owed.en.md:827 | 不相干 | C434 是设立 code-round-snapshot 格的账，「哈希与派腿时刻对不对得上靠人」仍真；14 轮补快照不改格的判别范围；引原话「ecords 新格 code-round-snapshot: 这次新写的, 按路径点名」 |
| F69 | .claude/kb/checks-owed.ja.md:311 | 不相干 | 译文同上：C436 讲归档删掉的路径被引用时两道都够不着，471 份归档不改它；引原话「。三层各自够不着：`archive-past-rounds.py` 的 `still_」 |
| F69 | .claude/kb/checks-owed.ja.md:350 | 不相干 | 译文同上：C449 里 base_of 说明写错变量名，与归档了几份无关；引原话「cripts/kb/archive-past-rounds.py` 的 base_of」 |
| F69 | .claude/kb/checks-owed.ja.md:392 | 要补 | 译文同上：C602 的积压那一批已归档之后补：2026-10-21 に再び 471 件が積み上がり（マイルストーン二・三とそれ以前の実験の記録）、GATE_BASE=1bb3a24dc の --apply --keep-registered でアーカイブ（records/2026-10-21-doc门禁违规修复.md）；「提交流程里单列一步」は未実装のまま；原话「riments 的 archive-past-rounds 格只判「工作区里不许留上一」 |
| F69 | .claude/kb/checks-owed.ja.md:814 | 不相干 | 译文同上：C437 gitlink 已修，与本阶段无关；引原话「cripts/kb/archive-past-rounds.py` 的 past_ro」 |
| F69 | .claude/kb/checks-owed.ja.md:827 | 不相干 | 译文同上：C434 设立 code-round-snapshot 格，哈希与派腿时刻靠人，仍真；引原话「ecords 新格 code-round-snapshot：这次新写的、按路径点名了产」 |
| F69 | .claude/kb/checks-owed.md:310 | 不相干 | C436 讲归档删掉的路径被引用时两道都够不着，471 份归档不改它；引原话「。三层各自够不着：`archive-past-rounds.py` 的 `still_」 |
| F69 | .claude/kb/checks-owed.md:349 | 不相干 | C449 里 base_of 说明写错变量名，与归档了几份无关；引原话「cripts/kb/archive-past-rounds.py` 的 `base_o」 |
| F69 | .claude/kb/checks-owed.md:389 | 要补 | C602 的「积压那一批已由提交 31128273（2026-10-10「归档上一轮研究记录」）归档」之后补：2026-10-21 又积压 471 份（里程碑二、三与更早实验的记录，m4- 起头的 0 份），照 GATE_BASE=1bb3a24dc 的 --apply --keep-registered 归档、22 份文件里 90 处引用只留文件名（records/2026-10-21-doc门禁违规修复.md）；「提交流程里单列一步」仍没有，现查 commit-stages.md、staged-gate.md 里 grep 归档 0 处。这一笔再次积压正是该欠账还开着的证据；原话「riments 的 archive-past-rounds 格只判「工作区里不许留上一」 |
| F69 | .claude/kb/checks-owed.md:815 | 不相干 | C437 gitlink 已修，与本阶段无关；引原话「cripts/kb/archive-past-rounds.py` 的 `past_r」 |
| F69 | .claude/kb/checks-owed.md:828 | 不相干 | C434 设立 code-round-snapshot 格，哈希与派腿时刻靠人，仍真；引原话「ecords 新格 code-round-snapshot：这次新写的、按路径点名了产」 |
| F69 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:52 | 事件句不改 | E155 页写 -stage2.out 2026-09-21 已随该格归档、用 git show 取回；那一天的事；引原话「**第二次跑（2026-09-19）第一段做了什么**：跑前」 |
| F69 | .claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:65 | 事件句不改 | E155 页同一段：stage2.out 2026-09-21 随归档格归档，那一次的事；引原话「riments 的 archive-past-rounds 格归档，用 `git sh」 |
| F69 | .claude/kb/milestone/03-third-txn.en.md:206 | 不相干 | 里程碑三待优化第 24 行：归档脚本删前查登记表点名的路径，状态「已关」（2026-10-12 现查），471 份归档不改它；引原话「24   Archive commit 31128273」 |
| F69 | .claude/kb/milestone/03-third-txn.en.md:232 | 事件句不改 | 第 50 行：2026-10-18 那一次归档改写变更史引用引起 history-loss 红；那一次的事，后来脚本改成跳过 history-carriers；引原话「cripts/kb/archive-past-rounds.py --apply`)」 |
| F69 | .claude/kb/milestone/03-third-txn.ja.md:206 | 不相干 | 译文同上：第 24 行归档脚本删前查登记表，已关；引原话「24   アーカイブコミット 31128273 が `.」 |
| F69 | .claude/kb/milestone/03-third-txn.ja.md:232 | 事件句不改 | 译文同上：2026-10-18 那一次归档改写引用，那一次的事；引原话「cripts/kb/archive-past-rounds.py --apply`）が」 |
| F69 | .claude/kb/milestone/03-third-txn.md:206 | 不相干 | 里程碑三待优化第 24 行：归档脚本删前查登记表点名的路径，已关，471 份归档不改它；引原话「cripts/kb/archive-past-rounds.py` 删之前查登记表点名」 |
| F69 | .claude/kb/milestone/03-third-txn.md:232 | 事件句不改 | 第 50 行：2026-10-18 那一次归档改写变更史引用，那一次的事；引原话「cripts/kb/archive-past-rounds.py --apply`）把」 |
| F69 | .claude/kb/tooling.en.md:77 | 不相干 | tooling.md 表：早于最近一轮的在下一次提交时归档，规则句仍真；引原话「periments archive-past-rounds slot)」 |
| F69 | .claude/kb/tooling.ja.md:77 | 不相干 | 译文同上：早于最近一轮的在下一次提交时归档，仍真；引原话「riments の archive-past-rounds スロット）」 |
| F69 | .claude/kb/tooling.md:77 | 不相干 | 译文同上：下一次提交时归档，仍真；引原话「riments 的 archive-past-rounds 格）」 |
| F69 | records/2026-09-16-subagent拆分提案.md:873 | 事件句不改 | 09-16 提案「改法三样，都在 archive-past-rounds.py」，那一次改的是脚本；引原话「cripts/kb/archive-past-rounds.py`：」 |
| F69 | records/2026-09-16-subagent拆分提案.md:943 | 事件句不改 | 09-25 的就地写清单里点名 archive-past-rounds.py，那一天的读数；引原话「32   终止进程的写法：会话里手敲的命令，与 `scr」 |
| F69 | records/2026-09-24-里程碑二收尾调度.md:236 | 事件句不改 | 09-24 同步远端冲突块的列表，那一次合并；引原话「.tsv、三个钩子、archive-past-rounds.py、agent-def-」 |
| F69 | records/2026-09-27-同步远端冲突块.md:1246 | 事件句不改 | 09-27 冲突块小节标题，那一次合并；引原话「cripts/kb/archive-past-rounds.py`（2 块）」 |
| F69 | records/2026-09-27-同步远端冲突块.md:1313 | 事件句不改 | 09-27 冲突块处理表，那一次合并；引原话「cripts/kb/archive-past-rounds.py`   1、2   补」 |
| F69 | records/2026-09-30-里程碑三开工第一轮.md:163 | 事件句不改 | 09-30 记录里「积压 3 559 份」是当日读数，用户当时的决定；引原话「cripts/kb/archive-past-rounds.py --apply`，单」 |
| F69 | records/2026-09-30-里程碑三开工第一轮.md:188 | 事件句不改 | 09-30 对三轮判决写完后跑 --apply 的安排，那一次的计划；引原话「cripts/kb/archive-past-rounds.py --apply 归档」 |
| F69 | records/2026-10-11-里程碑三待优化上下游协调.md:54 | 事件句不改 | 10-11 协调记录的分工表，那一次的计划；引原话「cripts/kb/archive-past-rounds.py`   tooling」 |
| F69 | records/2026-10-11-里程碑三待优化上下游协调.md:64 | 事件句不改 | 10-11 丁交回，归档脚本删前查门禁目录并加 --keep-registered，那一次的交回；引原话「cripts/kb/archive-past-rounds.py`（删前查门禁目录、`」 |
| F69 | records/2026-10-11-里程碑三知识腐烂治理.md:164 | 事件句不改 | 10-11 记录：archive-past-rounds 红不是这一批的，当日读数；引原话「（HEAD 就有）、archive-past-rounds（`research/` 下」 |
| F69 | records/2026-10-13-复用键与全仓改名.md:47 | 事件句不改 | 10-13 暂存 C434 新格并报 code-round-snapshot 真仓红一处，当日的事；引原话「ecords 新格 code-round-snapshot）。它报两处归我们的：① c」 |
| F69 | records/2026-10-13-复用键与全仓改名.md:54 | 事件句不改 | 同日主 agent 事后补记 m3-admission-r1 快照，当日处置；引原话「- 批 3 第三件交回：tooling-writer「新格」 |
| F69 | records/2026-10-13-复用键与全仓改名.md:81 | 事件句不改 | 同日改正快照哈希、重跑 9 份判决，当日处置；引原话「决「事后补记」那句，code-round-snapshot 格重跑 9 份判决都绿，已」 |
| F69 | records/2026-10-13-欠账还五条.md:211 | 事件句不改 | C434 交回及真仓误判的处置，那一次的交回；引原话「226d）：新格 `code-round-snapshot`，取新判决的 grep 抽」 |
| F69 | records/2026-10-13-欠账还五条.md:220 | 事件句不改 | 表里「code-round-snapshot 格真仓红一处」一行，那一天的处理状态，后来已事后补记；引原话「code-round-snapshot 格真仓红一处（m3-adm」 |
| F69 | records/2026-10-13-欠账还五条.md:430 | 不相干 | 说的是 code-round-snapshot 格仍读整份正文（规则句要跟着改），与这一格有没有补快照无关，仍真；引原话「判决形态处可补一句；code-round-snapshot 格仍读整份正文。三样进收拢」 |
| F69 | records/2026-10-13-欠账还五条.md:448 | 不相干 | C508 要跟着改的规则句与「code-round-snapshot 格仍读整份正文」，讲格读判决正文的范围，仍真，补 14 轮快照不改它；引原话「` 判决形态处）与 code-round-snapshot 格仍读整份正文   下轮开」 |
| F69 | records/2026-10-13-里程碑四预开发.md:832 | 事件句不改 | 10-13 记录 m4-i315-r1、r2 补登开工快照，当日处置；引原话「- 修红批验证第一步：本机 `m4-fix-red-stag」 |
| F69 | records/2026-10-17-全仓知识腐烂治理.md:77 | 事件句不改 | 10-17 记录里 archive-past-rounds 等格在 HEAD 上就红、落在别的会话未提交的实验上，当日分诊；引原话「- 批 6 收：写回员戊 46 处（含 D22 已定项 14」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:26 | 事件句不改 | 10-18 核实表：报告把 archive-past-rounds 归错一道，当日更正；引原话「成立（报告把 archive-past-rounds 归在这一道，实际在 doc」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:27 | 事件句不改 | 10-18 核实表里 archive-past-rounds 470 份是当日读数；引原话「links 8 处、archive-past-rounds 470 份；`disk_1」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:102 | 事件句不改 | 10-18 记录：归档脚本两处漏洞是当日报告的发现；引原话「cripts/kb/archive-past-rounds.py` 两处漏洞（A1 拼」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:118 | 事件句不改 | 10-18 批的文件清单，那一次的计划；引原话「cripts/kb/archive-past-rounds.py`」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:122 | 事件句不改 | 10-18 记录：archive-past-rounds 格 research/ 不在时整格退 77 是当日发现；引原话「riments 的 archive-past-rounds 格在 research/」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:127 | 事件句不改 | 10-18 批 3c 的派活，那一次的计划；引原话「只认登记的路径列、archive-past-rounds 格 research/ 不」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:137 | 事件句不改 | 10-18 归档脚本变异表只认两列的改动，那一次的交回；引原话「cripts/kb/archive-past-rounds.py`：变异表只认变异名与」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:138 | 事件句不改 | 10-18 记录：research/ 不在时已照「无对象可判」退 77，不改；当日审查回应；引原话「riments 的 archive-past-rounds 格：research/ 不」 |
| F69 | records/2026-10-18-审查报告核实与修复.md:148 | 事件句不改 | 10-18 出口验收里 archive-past-rounds 格退 77，当日读数；引原话「riments 的 archive-past-rounds 格照「没有 researc」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:14 | 事件句不改 | 开工现场里列提交之后按这次改动判的格要带 GATE_BASE=1bb3a24dc，是这一轮开工时的做法；引原话「unt-quota、code-round-snapshot、crates-advers」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:51 | 事件句不改 | 主 agent 定：基准取 1bb3a24dc、code-round-snapshot 照腿派出那天补快照，那一次的决定；引原话「只加了三语切换行」；code-round-snapshot 照腿派出那天的提交补快照、」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:69 | 事件句不改 | 进度快照：按时间写的「还开着」，随后 99、100 行写这几格做完；引原话「tch-scope、code-round-snapshot、archive-past-」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:82 | 事件句不改 | 开工现查：471 份上一轮记录、17 份判决没立开工快照，是这一轮开工那一天的读数；引原话「多登记 240 个；code-round-snapshot 17 份判决没立开工快照；」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:90 | 事件句不改 | 依赖表里主 agent 的任务行（照基准 --apply），这一轮的计划，做成见 99 行；引原话「5   archive-past-rounds 照基准 `--apply`」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:91 | 事件句不改 | 依赖表里主 agent 的任务行（补快照、判决旁写明事后补），这一轮的计划，做成见 100 行；引原话「6   code-round-snapshot 照腿派出那天的提交补快照、」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:99 | 事件句不改 | archive-past-rounds 照基准删了 471 份，就是这件事实的当日记录，那一次的操作；引原话「- archive-past-rounds：照 `GATE_BASE=」 |
| F69 | records/2026-10-21-doc门禁违规修复.md:100 | 事件句不改 | code-round-snapshot：14 轮派 clerk 事后补快照，另 3 轮归各自会话，就是这件事实的当日记录；引原话「- code-round-snapshot：17 份判决里 m4-i3」 |

### F70

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F70 | .claude/kb/checks-owed.en.md:750 | 不相干 | C 行讲钉表一行 22 段、红例账原行整行抄；钉表仍在，被撤的是两张已废红例账登记表，与此句无关；引原话「一行 22 段, 红例账原行整行抄并带档, 旧的八段」 |
| F70 | .claude/kb/checks-owed.ja.md:750 | 不相干 | 译文同上：钉表一行 22 段，被撤的登记表不在此句；引原话「` 一行 22 段、红例账原行整行抄并带档，旧的八段行」 |
| F70 | .claude/kb/checks-owed.md:751 | 不相干 | 钉表一行 22 段、红例账原行整行抄；钉表仍在，被撤的是两张登记表，与此句无关；引原话「` 一行 22 段、红例账原行整行抄并带档，旧的八段行」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:21 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是第二段红例账 151 行的产物说明；引原话「.tsv`：第二段的红例账，151 行（每个红的变体在」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:160 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是红例方向（第 7 列）的读法；引原话「线不同」，不分方向。红例账第 7 列带方向（`+` 是」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:161 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是红例账 151 行重放判定相同的核对；引原话「- 红例账 151 行，原实现与变体的」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:182 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是Q13 对 Z1 的读法；引原话「上红）。这些差别在红例账的反例状态上都是变体少了违例」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:257 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是红例账 161 行重放核对；引原话「红例账 161 行，原实现与变体的」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:286 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是产物清单里第二段红例账的行数；引原话「162   第二段红例账，每个红的变体在每个红了的域」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:317 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是Z5 界内读数里红例账 166 行的说明；引原话「（`table_row` 是冻结根里表的行号，对登记行号：2」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:319 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是SF3 见证里引红例账第 12、13 列；引原话「态各写一个见证**（红例账第 12 列原实现的逐态行、」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:321 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是形态一的见证；引原话「上变体 j4 红（红例账第 7 列 `+j4`）。登」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:327 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是见证里的原始 what 没取到；引原话「`what`**（红例账只存逐态行，变体的二进制跑完」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1007 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是PQ14 红例账的比较；引原话「ode=r2` 行。红例账（PQ14 的另一半，同一份」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1016 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是两份红例账域、N、序号的逐个相同；引原话「红例账两份都是 165 个反例状态」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1031 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是产物清单里 r2 模式的红例账；引原话「6   r2 模式的红例账（表头 1 行加 165 个」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1034 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是比对产物的说明；引原话「行数）与 PQ14 红例账的分列比」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1286 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是驱动每行带单条重放的命令；引原话「一条命令只补没跑的。红例账每行带单条重放的命令 `ta」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1364 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是留存说明：红例账只存逐态行；引原话「what` 没取到（红例账只存逐态行）。」 |
| F70 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1366 | 不相干 | E165 页这里的红例账是第二段产出的红例 tsv 产物（research/results 下），不是 .claude/gate.d 下被撤的登记表，说的是红例账方向只看首红状态的限制；引原话「- 红例账的方向只看首红那一个状态；整」 |
| F70 | .claude/kb/milestone/03-third-txn.en.md:206 | 事件句不改 | 里程碑三待优化第 24 行：归档提交 31128273 删了登记表点名的账，pinned-red-examples-complete 格从此每次红（2026-10-10 起），状态已关；那段时间的事，后来格改名为 pinned-red-examples-table 且登记表撤掉；引原话「de/gate.d/e165-void-red-example-l」 |
| F70 | .claude/kb/milestone/03-third-txn.ja.md:206 | 事件句不改 | 译文同上：第 24 行 2026-10-10 起格每次红的那段事；引原话「de/gate.d/e165-void-red-example-l」 |
| F70 | .claude/kb/milestone/03-third-txn.md:206 | 事件句不改 | 第 24 行 2026-10-10 起格每次红的那段事，状态已关；引原话「de/gate.d/e165-void-red-example-l」 |
| F70 | .claude/rules/verification.md:126 | 不相干 | rules/verification.md 这句已是结束那一版的写法（pinned-red-examples-table 只判钉表自己、不读红例账、产物与 research/；没有登记作废、被取代账的说法），无需再改；引原话「点名的变异还在，不读红例账、产物与 `research」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:920 | 事件句不改 | 09-30 记录：形式化验证真起 Verus 验收，其中钉全格没有登记作废的办法、派 tooling-writer 加登记表；当日读数与计划；引原话「- 形式化验证真起 Verus 验收（`~/<仓外目录>/s」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:921 | 事件句不改 | 09-30 记录：门禁两处交回，登记 r2-segment2 那份；当日的交回；引原话「358 格）；作废红例账登记表 `.claude/g」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:923 | 事件句不改 | 09-30 记录：两份 r2 档账在当日装置上定、派 tooling-writer 加被取代登记；当日的计划；引原话「lefs/work/e165-superseded-ledgers/`）给作废」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:924 | 事件句不改 | 09-30 记录：被取代登记交回，新表 e165-superseded-red-example-ledgers.tsv；当日的交回（该表后来已删）；引原话「lefs/work/e165-superseded-ledgers/repor」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:1015 | 事件句不改 | 09-30 记录：E7 报的形式化轻型红在 pinned-red-examples-complete，登记表第 11 行的账被归档删了；当日读数；引原话「- 实现 E7 交回（implementation-writ」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:1028 | 事件句不改 | 09-30 记录：E8 副本验证里形式化轻型红在登记表指着被删的账；当日读数；引原话「：形式化轻型红在 `e165-void-red-example-l」 |
| F70 | records/2026-09-30-里程碑三开工第一轮.md:1033 | 事件句不改 | 09-30 记录：主工作区轻型形式化 rc 1，只红 pinned-red-examples-complete；当日读数；引原话「mplete`（作废红例账登记表一行指着归档删掉的账，」 |
| F70 | records/2026-10-01-删除与并发事务.md:1488 | 事件句不改 | 10-01 记录：2026-10-10 形式化轻型在主工作区也红，登记表第 11 行的账被归档删掉；当日读数；引原话「plete 格「作废红例账登记表 .claude/ga」 |
| F70 | records/2026-10-01-删除与并发事务.md:1570 | 事件句不改 | 10-01 记录：规格里「形式化红例账缺失」等欠账的写回；当日的记录；引原话「0.86%、形式化红例账缺失）   写回过规格检查、」 |
| F70 | records/2026-10-01-删除与并发事务.md:1589 | 事件句不改 | 10-01 记录：形式化红例账缺失在待优化第 24 条，是当日的分流说明；引原话「第 26 条；形式化红例账缺失在第 24 条。剩 I-」 |
| F70 | records/2026-10-01-删除与并发事务.md:1596 | 事件句不改 | 10-01 记录：提交前两处会红的修法（从父提交取回 kill-bounds.out）；当日处置；引原话「接测」修：E165 红例账配的 `e165-repla」 |
| F70 | records/2026-10-01-对接上游测试脚本规范.md:76 | 事件句不改 | 10-01 记录：H6 钉表整行抄出红的那一行，处置「不够，记欠账」；当日；引原话「件:行号」，⑦⑧ 是红例账第 12、13 列   不够」 |
| F70 | records/2026-10-01-里程碑三形式化接入.md:65 | 事件句不改 | 10-01 记录：用户补一句用门禁控制红例；当日的决定；引原话「场把这一趟的红例写成红例账（哪一趟、域、判据、输入）并」 |
| F70 | records/2026-10-01-里程碑三形式化接入.md:80 | 事件句不改 | 10-01 记录：执行员的驱动脚本被写范围闸挡、红例账 151 条重放判定相同；当日的交回；引原话「- E165 对抗一族入库与第二段交回（草稿 `/tmp/c」 |
| F70 | records/2026-10-01-里程碑三形式化接入.md:114 | 事件句不改 | 10-01 记录：门禁任务表里「红例账两格」；当日的计划；引原话「定行，门禁要认得）与红例账两格；照 `.claude/」 |
| F70 | records/2026-10-01-里程碑三形式化接入.md:365 | 事件句不改 | 10-01 记录：脚本改成认出红例账那一趟的名字；当日的处置；引原话「- 6-2 交回（草稿 `/tmp/claude-1000/」 |
| F70 | records/2026-10-01-里程碑三形式化接入.md:368 | 事件句不改 | 10-01 记录：门禁那一格的现跑与留下的一处；当日的交回；引原话「根>`：「✓ 2 份红例账 312 行都钉在 … 2」 |
| F70 | records/2026-10-01-里程碑三形式化接入.md:1397 | 事件句不改 | 10-01 记录：回根环线约定「钉表各行与红例账逐字节相同」；那条线当时的约定；引原话「各行从第 9 段起与红例账逐字节相同，不能改；③ 按条」 |
| F70 | records/2026-10-11-里程碑三待优化上下游协调.md:64 | 事件句不改 | 10-11 协调记录：丁交回，登记表第 11 行点名的账已被删（交用户定）；当日读数；引原话「- 丁交回（`m3-pending-b3-ding/repo」 |
| F70 | records/2026-10-11-里程碑三待优化上下游协调.md:67 | 事件句不改 | 10-11 协调记录：kb 第二份交回，抽查 03 第 24 行换上的产物名；当日的交回；引原话「行换上的产物名（`e165-void-red-example-l」 |
| F70 | records/2026-10-11-里程碑三待优化上下游协调.md:69 | 事件句不改 | 10-11 协调记录：路径与结论登记行号改正交回，同属第 24 行那一类；当日的交回；引原话「- 路径与结论登记行号改正交回（`m3-pending-b3」 |
| F70 | records/2026-10-11-里程碑三待优化上下游协调.md:117 | 事件句不改 | 10-11 协调记录：用户弹窗定删登记行，主 agent 删了作废登记表第 11 行；当日的处置；引原话「de/gate.d/e165-void-red-example-l」 |
| F70 | records/2026-10-11-里程碑三知识腐烂治理.md:129 | 事件句不改 | 10-11 记录：钉表 complete 格在 HEAD 上红，归档删了产物；当日读数；引原话「out`，与它配对的红例账还留着；乙组在副本里从 `3」 |
| F70 | records/2026-10-12-里程碑三非代码收尾.md:44 | 事件句不改 | 10-12 记录：课题 8 第 24 行现查已关，登记表那一行在暂存区已撤；当日读数；引原话「de/gate.d/e165-void-red-example-l」 |
| F70 | records/2026-10-13-里程碑四预开发.md:306 | 事件句不改 | 10-13 记录：形式化轻型两格红与基线副本同形（副本缺红例账）；当日读数；引原话「fs` 一族、副本缺红例账）。写段只打不验：备份 `m」 |
| F70 | records/2026-10-18-审查报告核实与修复.md:61 | 事件句不改 | 10-18 记录：主工作区形式化轻型 pinned-red-examples-complete 红，交用户；当日读数，同批后面已换成 table 格；引原话「omplete 红：红例账 `research/res」 |
| F70 | records/2026-10-18-审查报告核实与修复.md:117 | 事件句不改 | 10-18 记录：分工表里甲簇 E165 钉表的改法；那一次的计划；引原话「表自己，钉全改在产出红例账的那一趟当场判；`--che」 |
| F70 | records/2026-10-18-审查报告核实与修复.md:123 | 事件句不改 | 10-18 记录：甲簇交回「作废、被取代两张红例账登记表删掉」，就是这件事实当日的交回；引原话「跑；作废、被取代两张红例账登记表删掉；`verific」 |
| F70 | records/2026-10-18-审查报告核实与修复.md:134 | 事件句不改 | 10-18 记录：commit-stages.md 第 32 行指纹路径清单去掉两张已删的红例账登记表，当日的改动（现查 commit-stages.md 第 6 步清单已无这两张）；引原话「径清单去掉两张已删的红例账登记表与 `research」 |

### F72

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F72 | .claude/kb/INDEX.md:28 | 要改 | 内容列改成：翻译术语表：中、英、日三语并排，仓根 README、`briefs/` 与 `.claude/kb/` 的三语版本照它用词（INDEX.en.md 与 INDEX.ja.md 第 28 行同句，候选没罩到，见 M1 M2）；原话「译术语表：中、英、日三语并排，仓根 README 与」 |
| F72 | .claude/kb/milestone/02-second-txn.en.md:472 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），小节「全量（2026-09-30）」仍在该文件 ### 标题里；原话「- **Status (landed)**: All fou」 |
| F72 | .claude/kb/milestone/02-second-txn.ja.md:472 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md`「全量（2026-09-3」 |
| F72 | .claude/kb/milestone/02-second-txn.md:7 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md）；同一行后半句「这一版的简报是 briefs/2026-09-28.md」已是新路径；原话「2026-09-28.zh.md`「全量（2026-09-3」 |
| F72 | .claude/kb/milestone/02-second-txn.md:486 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md`「全量（2026-09-3」 |
| F72 | .claude/kb/milestone/03-third-txn.en.md:50 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md` section "Ful」 |
| F72 | .claude/kb/milestone/03-third-txn.en.md:113 | 要改 | 把 `briefs/2026-10-10.zh.md` 改成 `briefs/2026-10-10.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），小节「这个结果不能读成「验过了、没有红」」仍在该文件 ### 标题里；原话「V·7   Pinned values of known」 |
| F72 | .claude/kb/milestone/03-third-txn.en.md:141 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md）（这一行两处）；原话「- **Existing in repository**:」 |
| F72 | .claude/kb/milestone/03-third-txn.ja.md:50 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md`「フル実行（2026-09」 |
| F72 | .claude/kb/milestone/03-third-txn.ja.md:113 | 要改 | 把 `briefs/2026-10-10.zh.md` 改成 `briefs/2026-10-10.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「五·7   マイルストーン二収束時に残された既知の赤の固」 |
| F72 | .claude/kb/milestone/03-third-txn.ja.md:141 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md）（这一行两处）；原话「- **リポジトリ内の既存要素**：クラッシュポイントはレコ」 |
| F72 | .claude/kb/milestone/03-third-txn.md:50 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md`「全量（2026-09-3」 |
| F72 | .claude/kb/milestone/03-third-txn.md:113 | 要改 | 把 `briefs/2026-10-10.zh.md` 改成 `briefs/2026-10-10.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「五·7   里程碑二收尾留下的已知红钉值（用户 2026」 |
| F72 | .claude/kb/milestone/03-third-txn.md:141 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md）（这一行两处）；原话「- **仓里已有的**：崩溃点由记录器按屏障段切（`crat」 |
| F72 | .claude/kb/milestone/04-fourth-txn.en.md:74 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md` section "Ful」 |
| F72 | .claude/kb/milestone/04-fourth-txn.ja.md:74 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md`「フル実行（2026-09」 |
| F72 | .claude/kb/milestone/04-fourth-txn.md:74 | 要改 | 把 `briefs/2026-09-28.zh.md` 改成 `briefs/2026-09-28.md`（现查 ls briefs：只有 .md、.en.md、.ja.md，没有 .zh.md，1bb3a24dc 起 .zh.md 已改名为 .md），节名不变；原话「2026-09-28.zh.md`「全量（2026-09-3」 |
| F72 | .claude/kb/milestone/04-fourth-txn.md:145 | 不相干 | 这一格说对照组数据写进 README 与当期简报并三语一起改，仍真；kb 也三语不改它；引原话「对照组的数写进仓根 README 与当期简报（`briefs」 |
| F72 | .claude/kb/translation-glossary.md:3 | 不相干 | 这句已是新约定：登记在 .claude/trilingual-documents 的文件，中文是默认版本、.en.md 英语、.ja.md 日语，且已写 kb；引原话「**仓根 README、`briefs/` 与 `」 |
| F72 | .claude/skills/trilingual/SKILL.md:3 | 不相干 | skill 说明已是新约定，射程写「全部文档」含 kb；引原话「glefs 里登记为三语的文件（仓根 README、」 |
| F72 | CLAUDE.md:102 | 不相干 | CLAUDE.md 这一行已写「README、简报与 kb 的译文照它用词」，已含 kb，无需改；引原话「译术语表：中、英、日三语并排，README、简报与」 |
| F72 | records/2026-10-13-三语README与简报.md:1 | 事件句不改 | 记录标题，写的是 2026-10-13 那一批的名字；引原话「# 三语 README 与简报、三语」 |
| F72 | records/2026-10-13-三语README与简报.md:9 | 事件句不改 | 该记录第 9 行写的是 2026-10-13 那一批的做法：.md 是英语（默认）、.zh.md 是中文；后来 1bb3a24dc 反转，不回改当日记录；引原话「1. 仓根 README 与 `briefs/` 下」 |
| F72 | records/2026-10-13-三语README与简报.md:17 | 事件句不改 | 该记录任务表里的第一件，当日的计划；引原话「doc-text 三语格、样本、归属表行、gove」 |
| F72 | records/2026-10-13-三语README与简报.md:27 | 事件句不改 | 该记录说当日把中文原文复制成 README.zh.md，当日的做法；引原话「文原文原样复制成 `README.zh.md` 与 `bri」 |
| F72 | records/2026-10-13-三语README与简报.md:32 | 事件句不改 | 该记录说 2026-09-28 简报的标题让某一道红，原样挪进 .zh.md，当日的事；引原话「道红，原样挪进了 `.zh.md`；这一道在 `record」 |
| F72 | records/2026-10-13-复用键与全仓改名.md:53 | 事件句不改 | 别的会话的提交 f0b94fef（英语为默认、中文挪到 .zh.md），当日的事；引原话「f0b94fef（「README 与简报改为三语：英语为默认」 |
| F72 | records/2026-10-13-复用键与全仓改名.md:1004 | 事件句不改 | 用户 2026-10-18 补定的原话与当日做法：简报 .md、.zh.md、.ja.md 三份，当日的事；引原话「0-18.md`、`.zh.md`、`.ja.md` 三份（」 |
| F72 | records/2026-10-13-复用键与全仓改名.md:1052 | 事件句不改 | 当日提交前暂存区清单，列三语简报，当日读数；引原话「用户定（原话）：「开始走 提交流程 跑提交门禁 你不需要再跑」 |
| F72 | records/2026-10-13-复用键与全仓改名.md:1059 | 事件句不改 | 当日补简报与 README 的记录，当日的事；引原话「已定项 17 等）、三语简报 `briefs/202」 |
| F72 | records/2026-10-13-复用键与全仓改名.md:1062 | 事件句不改 | 当日补暂存区与三语简报的说明，当日的事；引原话「，只暂存那一块；② 三语简报各加一节「提交时六道重型」 |
| F72 | records/2026-10-17-全仓知识腐烂治理.md:64 | 事件句不改 | 批 5 主 agent 自己改的清单，列三语 README 的 crate 表，当日的事；引原话「择根、扫环三截的核；三语 README 的 crat」 |
| F72 | records/2026-10-17-全仓知识腐烂治理.md:72 | 事件句不改 | 批 5 派回扫员，R3 通读三语 README，当日的派活；引原话「CLAUDE.md、三语 README、INDEX、」 |
| F72 | records/2026-10-17-全仓知识腐烂治理.md:74 | 事件句不改 | 批 5 自检收口里 doc-text 三语两格红在别的会话的简报上，当日读数；引原话「date-once、三语两格）落在别的会话 10-1」 |
| F72 | records/2026-10-17-全仓知识腐烂治理.md:95 | 事件句不改 | 课题 3 现数改写清单，列三语 README，当日的事；引原话「ion-build、三语 README、欠账表、里程」 |
| F72 | records/2026-10-19-kb三语门禁与skill.md:5 | 事件句不改 | 用户原话：README/简报命名约定（.md 是英语）已变、现在默认是中文；是当日的指令，记录这一批的缘起；引原话「b下面的文档 要更新三语 这个加skill 和门禁」」 |
| F72 | records/2026-10-19-kb三语门禁与skill.md:28 | 事件句不改 | 该记录「现查」：当时 LANGUAGES 与登记表、CLAUDE.md 还写旧约定；是当日现查，现 CLAUDE.md 第 101 行已写「YYYY-MM-DD.md 是中文（默认）」；引原话「- 现查：README 与简报在提交 `1bb3a」 |
| F72 | records/2026-10-19-kb三语门禁与skill.md:32 | 事件句不改 | 该记录写当日改样本到新约定，当日的做法；引原话「re-green 补三语切换行与链同一种语言的简报。」 |
| F72 | records/2026-10-19-文档普查报告核实与修复.md:37 | 事件句不改 | 文档普查记录里 trilingual-files 报缺 README.zh.md 的当日读数；引原话「files 报缺 `README.zh.md`、四」 |

### M（候选之外，第 10 步反向查到的）

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| M1 | .claude/kb/INDEX.en.md:28 | 要改 | 内容列改成：Translation glossary: Chinese, English, and Japanese side-by-side; trilingual versions of the root README, `briefs/` and `.claude/kb/` follow its terms；依据：INDEX.md:28 同句候选里判要改（kb 译文已纳入三语），译文行的检索词是中文、候选没罩到，现查 sed -n 28p 原样输出见上 |
| M2 | .claude/kb/INDEX.ja.md:28 | 要改 | 内容列改成：翻訳用語集: 中国語・英語・日本語の三言語を並記、ルート README、`briefs/` および `.claude/kb/` の三言語版はその用語に従う；依据同 M1 |
| M3 | .claude/gate.d/lib-governance-refs.py:74 | 要改 | README_PATTERNS 还是 ["README.md", "README.zh.md", "README.ja.md"]，第 73 行注释还写「英语 README.md 是默认版本，README.zh.md 是中文」；现约定是 README.md 中文、README.en.md 英语（ls README* 现查：README.en.md、README.ja.md、README.md），所以治理引用那一格实际扫不到 README.en.md、README.zh.md 查无此文件被跳过；同一旧说法还在该库第 2、5、44、53、460 行与 .claude/gate.d/doc-registries.sh 第 35、69、209、210、286 行。改成 ["README.md", "README.en.md", "README.ja.md"] 并改文字与样本，交 tooling-writer；这是门禁代码不是现状载体，候选表不罩，我没判它的样本期望 |

## 没做什么

- 没改任何仓内文件，没改 `stale=`，没读会话记录，没碰 `~/.claude/`；没跑 gate 或重型测试。
- 候选表原文列只有前 500 字：超出的部分我回仓里读了原行，但没有把每一行的整行都抄进理由，理由里引的是前 500 字内的原话。
- 事件句不改的 125 行我按「记录里按时间写的那一天的事」判，没有逐条去 `git log` 核每一天的数；判错的话推翻条件是：其中某句其实在说今天的样子（有没有、几条、还欠什么），且被这一阶段改了。我特别回看过的有 F68 第 38 条（`records/2026-10-21-doc门禁违规修复.md:69`「还开着、这一轮接着做的」，按时间写在「## 经过」里，下一条 71 行就是交回）和 F69 第 41 条（`records/2026-10-13-欠账还五条.md:220`「code-round-snapshot 格真仓红一处」，同日已事后补记）。
- F72 的 16 行路径引用我只核了路径与小节标题都在新文件里（`briefs/2026-09-28.md` 第 74 行、`briefs/2026-10-10.md` 第 89 行），没核小节里的数字与被引句子是否仍一致。
- M3 只列了 `README.zh.md` 的残留，没有读 `lib-governance-refs.py` 的全部逻辑，也没跑它的样本。
- 事实表写事实表那一位点出的现状矛盾（F45、F9、F59、F31、F46、F73）不在我的组；我的组里没有碰到同类现状句。

## 自检

```
$ python3 scripts/kb/stale-candidates.py --check-report candidates.tsv report.md --groups F67,F68,F69,F70,F72   （--groups F67-F72 被拒，因为候选表没有 F71）
  ✓ 报告判全了：221 行候选都有逐行判定
退出码 0
$ grep -cE "^\| F(67|68|69|70|72) \| " report.md   # 判定行总数
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 要改 \|" report.md
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 要补 \|" report.md
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 事件句不改 \|" report.md
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 不相干 \|" report.md
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 要人看 \|" report.md
$ grep -cE "^\| M[0-9] \| " report.md
```

## 判定计数

上一节里那几条 grep 因输入文件同时是输出文件（grep 拒绝）没出数，这里对报告的副本重数：

```
$ grep -cE "^\| F(67|68|69|70|72) \| " report-copy.md   # 判定行总数
221
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 要改 \|" report-copy.md
17
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 要补 \|" report-copy.md
3
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 事件句不改 \|" report-copy.md
125
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 不相干 \|" report-copy.md
76
$ grep -cE "^\| F(67|68|69|70|72) \| [^|]+ \| 要人看 \|" report-copy.md
0
$ grep -cE "^\| M[0-9] \| " report-copy.md
3
```

合计：要改 17、要补 3、事件句不改 125、不相干 76、要人看 0，共 221 行；另 M 行 3 行（均要改）。
