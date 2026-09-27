# kb 写回报告（第六批）

## 规格文件

- `/tmp/claude-1000/kb-batch6-drafter/spec-jia.json`（7 条，先施加）
- `/tmp/claude-1000/kb-batch6-drafter/spec-yi.json`（8 条，后施加）

主 agent 对起草员报告「要主 agent 判的点」四条的裁决（本批遵照）：
第 1 条（D16（发布语义） 已定项 7 射程里旧「fsync 等根槽持久之后才返回」那条 bullet）本批不改，主 agent 落完之后自己改；
第 2、3 条本批不改；第 4 条（甲乙两份规格谁先写、变更史「其N」谁在最上面）照规格插，不需要判。

## 开工时哈希（施加任何改动之前）

```
2344da7ebb42db316a89a01cd885f3b95b2b79cbe3201ac9391cdb12cdab709f  .claude/kb/decisions/16-发布语义.md
fbee4b907c42c2505718d873e3282a9e30986d80582157f04555f59cd3c94f46  .claude/kb/decisions/23-journal的角色与格式.md
2ef1055cb9f1ad2bd813777e890254d4dc3bed48ab4d94b3f0aae37b3b329027  .claude/kb/decisions/18-块里携带什么信息.md
e5cb983dfff4ef764d7576e6e5f181f1e060d1f72e7a943609134c15a90d83bb  .claude/kb/checks-owed.md
a4bebfdb5cb2fb20e4120a878e14d592e75caa57572cad1ec74d5d7f037c6b4e  .claude/kb/decisions-history/2026-09.md
0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895  .claude/kb/decisions.md
d48c37aa1d61886dcb89de6c59686aff3fcc0606c340c482a2a34a69f9988eb0  .claude/kb/decisions-history.md
3532cb98c1747408e4ab69247dc88b30c77d4d6998905a61f468b71f8048c2a9  .claude/kb/decisions/22-单元原子性怎么合成.md
4a880955a551193bdfe9ba8bd1e6baa9659bd9fef314efbd0eb7f494643fb010  .claude/kb/decisions/09-加密.md
a33bb1ed3adb467412ad0bc5156571e719861c7fb5a0882c3389d976c540394f  .claude/kb/decisions/15-格式冻结政策.md
122c73f819935cb45685119d834795f57f6792ca2682c8ee883c0395553cd698  .claude/kb/invariants.md
```

（文件当时已有别的会话没提交的改动，此处照记；`.claude/kb/decisions.md` 全程未被本批规格点名，收尾时哈希与开工时一致。）

## 写范围闸预检

9 个目标路径逐个喂 `write-guard.sh`（`AGENT_HOOK_DETECTIONS` 指到 `/tmp/claude-1000/kb-batch6-scribe/precheck.jsonl`），全部退出码 0，无拒。

## 「其N」现查

写第一行之前查 `.claude/kb/decisions-history/2026-09.md` 2026-09-27 当天最大的「其N」：
`grep -n '^### 2026-09-27' ...` 最上面一条是「（其十三）」，之下依次其十二、其十一……其一，没有其十四及以上。
规格取的其十四～其十七未被占用，不需要顺延、不需要改号对照。

## replace-batch.py 原样输出

```
$ python3 research/scripts/replace-batch.py --dry-run spec-jia.json
✓ 7 处都命中得对（5 个文件），--dry-run 没写

$ python3 research/scripts/replace-batch.py spec-jia.json
✓ 7 处替换，写了 5 个文件（改名换上新 inode），回读一致

$ python3 research/scripts/replace-batch.py --dry-run spec-yi.json
✓ 8 处都命中得对（5 个文件），--dry-run 没写

$ python3 research/scripts/replace-batch.py spec-yi.json
✓ 8 处替换，写了 5 个文件（改名换上新 inode），回读一致
```

## 决策变更史：49 号 --write

`bash .claude/gate.d/49-history-brief.sh`（不带 --write）先跑：只报「`.claude/kb/decisions-history.md` 的生成块与按原文重新生成的不一致」（预期中的陈旧，因为月文件已经改了、汇总文件还没重生成），没有列出任何「快查·… 还没写」的具体条目，也没有点名不是这一轮的条目。跑前记 `decisions-history.md` sha256（`d48c37aa1d…`，与开工时一致），跑
`bash .claude/gate.d/49-history-brief.sh --write`：
```
✓ 按原文重新生成了 .claude/kb/decisions-history.md：576 条条目，这次补了 0 条「（待补）」快查
```
跑后 `diff` 前后两份文件：新增的行只对应本批四条新条目（D22/D9/D15 组合出现在 D22、D9、D15 三张卡片各一次，D16 一次，D23/D18 组合两次，invariants.md 一次），被挤掉的旧行是「每张卡片只留最近 3 次」的既定设计，没有出现不属于这一轮的新增行。

## kb 门禁（doc-lint、20、21、22、30、36、43、49、75、78、97）原样末行

```
doc-lint:  ✓ 文档铁律检查通过（检查 524，跳过 0；DOC_LINT_VERBOSE=1 看全部）
20:        ✓ kb 形状检查通过（kb 文件 212 份、规则 7 份）
21:        ✓ 决策分项清单与正文同步（350 个分项）
22:        ✗ 分项引用与正文状态不一致 1 处 —— 见下「本批带来的红」
30:        ✓ 决策正文改了 47 行，变更史新增 15 条条目
36:        ! 本次无对象可判：扫了 207 份 kb 文件（invariants.md 与变更史之外），没有一处写「N 条在用」的声明（退出码 77，本次未跑，非通过）
43:        ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 562 行，2 张表）
49:        ✓ 按原文重新生成了 .claude/kb/decisions-history.md：576 条条目，这次补了 0 条「（待补）」快查
75:        ✓ 决策与实验双向登记对得上、回看不过期（查了实验页 162 个、决策 28 条；已定项 345 个其中「无实验」147 个、表行 801 行、这次改动触发回看 10 处）
78:        ✓ 已还清的行引的测试名都还在（83 行，查了 45 个标识符）
97:        ✓ 这次改动新写或改写的 3 行不变量都点名了已定分项（基准 faf255e2）；存量 79 行里 34 行还点不出（记在 C69）
```

## 本批带来的红（22 号，停下交回，未自行修改规格原文）

`bash .claude/gate.d/22-item-ref-status.sh`：

```
✗ 分项引用与正文状态不一致 1 处
   .claude/kb/decisions-history/2026-09.md:34 「已定项 17」归属判不出：- 改前：I-2.4（头校验和覆盖范围） 的不变量列只说「那 29 字节由『恒 0、读者遇到非 0 一律判该结构损坏』这条规则守（已定项 1
```

现查：这句「已定项 17」是 spec-yi.json 第 6 条（invariants.md I-2.4 那一行）改动之前的旧串原文，规格把它逐字抄进了变更史「其十七」条目的「改前」bullet（spec-yi 第 8 条，`.claude/kb/decisions-history/2026-09.md`）。它在 invariants.md 原文里能判对（22 号沿着行内更早出现的「D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18」把归属判成 D18，18 号确有已定项 17），但同一句话被搬进变更史条目之后，22 号在这个文件里往回找的「上一次出现过的 D 号」落在更早一条「其十四」条目里提到的 D25（目标负载优先级），而 D25 没有第 17 条分项，判不出归属。
这不是规格与原文对不上（旧串确实唯一命中、依据也配对），是规格逐字搬运原文之后在新的上下文里触发了 22 号的一处误判；kb-scribe 定义要求「照规格写，不补内容」，不能自行往这句「已定项 17」前面加「D18（块里携带什么信息）」这几个字——那是规格没写的句子。停在这里交回，不自行改规格原文，请主 agent 判是否要在正文里把这句补成「D18（块里携带什么信息） 已定项 17」或改写规格。

## 收尾时哈希

```
6a4e48c7a92ebb54f83f0c24afae9da4a208fbac4ef1312d1b52c144fdf5ee33  .claude/kb/decisions/16-发布语义.md
609f1a2e5d437759392cbd244910fbe7a8d61acea849de96965944f3e3007028  .claude/kb/decisions/23-journal的角色与格式.md
9175579c6773e9c6b84dec1b8b437793e54d70e78b7322245d696e32e925ddc8  .claude/kb/decisions/18-块里携带什么信息.md
da5f27bb8d8865131e2744f4aad57518627c28fb31f29c6e29b0f61ab62f21fe  .claude/kb/checks-owed.md
c44df3dcefb6772d74b72d2830f7a1a391135b4a47d89e24e7cfc321c8a8412a  .claude/kb/decisions-history/2026-09.md
0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895  .claude/kb/decisions.md （未改动）
a3ed0ab086d485bccc72b2cc2a428513141a733802b426d47a22ce92e54deeb4  .claude/kb/decisions-history.md
79e893c3899f42e19e7dbb569a2b0755abc70a9d1f578146cd6f34f4da98696e  .claude/kb/decisions/22-单元原子性怎么合成.md
225b39b7892d1a00143e6fbb0ff82d11453e44fb22d7602a8c51f1896a54f703  .claude/kb/decisions/09-加密.md
73d736c175a1d6a0464096bf8e977c3d25d8378a6085462771f5bfa8fb44d442  .claude/kb/decisions/15-格式冻结政策.md
eb2bcc497597f69027251032ec3f7bd484439c752705e78c1135e7ed8c18104a  .claude/kb/invariants.md
```

## 没做什么

- 没判定案对不对，只照两份规格逐字写。
- 没跑重型测试。
- 没修 22 号那处红（这一批带来的，但修法是往正文加规格没写的字，停下交主 agent 判）。
- 没管 `.claude/kb/layout/01-first-txn.md`：dry-run 时 27、52 号钩子提示报的红点名这份文件，`git status --porcelain` 显示它是别的会话没提交的改动，不在本批规格里，没有动它。
- 36 号退出码 77（本次未跑，非通过），不算判过。
- 未跑 `.claude/gate.d/stage-owners.tsv` 里登记给 kb-scribe 的全部 41 道阶段——主 agent 这次任务消息明确点名了要跑的门禁清单（doc-lint、20、21、22、30、36、43、49、75、78、97），照这份窄清单执行。
