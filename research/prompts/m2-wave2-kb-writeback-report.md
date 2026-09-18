# kb-scribe 报告：m2-wave2-kb-writeback

规格：`research/prompts/m2-wave2-kb-writeback-spec.md`（第一至四节改动、第五节门禁）。

## 写范围闸（开工前）

四个目标路径逐一喂 `write-guard.sh`，全部 exit=0：
`.claude/kb/milestone/02-second-txn.md`、`.claude/kb/checks-owed.md`、
`.claude/kb/decisions/08-核心索引结构.md`、`.claude/kb/layout/01-first-txn.md`。

## 开工时 sha256sum（规格点名文件 + 当月变更史 + 决策文件；这几个文件当时已带别的会话未提交的改动，`git status --porcelain` 显示为 M/A）

```
f7d71731a24d1221aac770608ed19874de5c45500e7b09b27bd14b4d07eecd77  .claude/kb/milestone/02-second-txn.md
c97c28428085a99a6bd698fff4801b723051b8b29234cac2340c6cf28e012287  .claude/kb/checks-owed.md
a1b478fbe610cef65629df47bad16a66ef9af5e073b3f89740cf057a811a7271  .claude/kb/decisions/08-核心索引结构.md
f912d445a69c5a5e8fa76b7a4967ea652d9c4bedaa84f30521d17137ddfc9038  .claude/kb/layout/01-first-txn.md
e517045b88a26395e400dd5fcee1087e5b3b2162b6780ceda6de52a36b3521ac  .claude/kb/decisions-history/2026-09.md
e56c93f96a4aea8ca2f6b01fef6e795c5c83a6bf4f492e6564d569a3f2b70a31  .claude/kb/decisions.md
a70d60ec9732a93866e4feb59dcc92a6b41bc369ead0163950f9d2a63a0a0c16  .claude/kb/decisions-history.md
```

规格依据「不记决策变更史（决策定义都没变）」，本轮未写 `decisions-history*`，收尾复核三份哈希与开工时一致（见文末）。

## replace-batch.py 规格与执行

规格文件：`/tmp/claude-1000/kb-m2-wave2-writeback/spec.json`（11 处替换，覆盖 4 个文件；由
`/tmp/claude-1000/kb-m2-wave2-writeback/build_spec.py` 生成，避免手写 JSON 转义出错）。

`--dry-run` 原样输出：
```
✓ 11 处都命中得对（4 个文件），--dry-run 没写
```

不带 `--dry-run` 原样输出：
```
✓ 11 处替换，写了 4 个文件，回读一致
```

11 处替换与规格的对应关系：
- 一之 1（row ②「项」列末尾追加）、一之 2（row 11「去向」列整体替换）、
  一之 3（row 20b，落在「性质」列，按规格原话原样替换整段，不挪列）、
  一之 4（row 20c「去向」列替换）、一之 5（row 28「去向」列末尾追加）
- 二（在收口表第 36 行整行之后插入第 37–40 行，按规格代码块逐字追加）
- 三（在欠账表 C380 整行之后插入 C381 整行，按规格代码块逐字追加）
- 四之 1（`08-核心索引结构.md` 第 429、430 行两处替换）
- 四之 2（`01-first-txn.md` 字节表一行替换 + 「## 历史版本」下插入「### 2026-09-18」一节，
  紧接在原「### 2026-09-17（搬目录）」之前）

全部回读核对与规格逐字一致（已用 `sed -n` 分别核对每一处，见下方门禁结果部分的附带核实）。

## relabel-item.py

规格未给任何「分项号 + D<n> <k>」的翻状态指令（规格标题下的依据只涉及收口表五行改写、加四行、欠账表加一行、决策正文与字节表四处；文末「另外两条不在规格文件里的约束」明确「收口表第 37、38 行的『去向』写的是『实现员在改』，照写，之后由主 agent 另给规格翻状态」）。据此本轮不跑 `relabel-item.py`，不涉及步骤 3 的写范围（`records/**/*.md`、`research/**/*.md`、`research/**/*.rs`、`.claude/rules/*.md`）。

## 门禁：规格第五节点名的六项

```
### 10-kb-rot.sh
  ✓ kb 腐化审计通过
EXIT=0
### 26-number-name-sync.sh
  ✓ 编号与简称一致（扫 224 个文件，prompts/ 按证据链原样保留）
EXIT=0
### 67-milestone-closeout-owed.sh
  ✓ 里程碑收口表收全了文件里点名、还开着的欠账号（判了 1 份；02-second-txn.md：全文点名 55 个 C 编号，其中还开着 50 个：表里 43 个、豁免 7 个）
    没判 1 份（没有收口表标记）：01-first-txn.md
EXIT=0
### 32-first-txn-fields.sh
  ✓ 第一个事务的字段表指向都成立（查了 356 处引用）
EXIT=0
### 38-field-table-projection.sh
  ✓ 被投影的分项，字段表都投影全了（3 张字段表、24 行；投影表引用了 102 条分项）
EXIT=0
### 42-first-txn-trio.sh
  ✓ 第一个事务的三份文件互相挂钩（判「是」的未定项 0 条、字节表 11 节、里程碑 8 步）
EXIT=0
```

`32`、`38`、`42` 号均绿，未撞上「字节表偏移 88 那一格的写法」红——照另给的约束，这条不适用，原样报告本次结果。

### doc-lint.sh（规格第五节点名）

```
✗ 编号 Z1 出现 8 次，却没有任何登记位
✗ 编号 Z2 出现 7 次，却没有任何登记位
✗ .claude/kb/milestone/02-second-txn.md  1 处编号引用没带简称或简称不符
    :357  C381 裸引用 —— | 40 | 发布在最后一步（超级块槽）失败时根已 FUA 落盘，分配器照样退回：同一个写入口换发一次空发布、再断电之后
✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、3 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 433，跳过 0）
EXIT=1
```

红全部落在这一轮写入的句子上：Z1/Z1-a/Z1-d/Z1-b/Z1-f（第 37、39 行）、Z2/Z2-c/Z2-e（一之 3、二第 40 行、三 C381 行）、
第 357 行末列裸写的「C381」（二第 40 行「记在哪」列）——这些都是规格第二节代码块与第三节 C381 行要求「整行照抄」的原文，
规格本身没有给 Z1/Z2 登记位或给「C381」补简称。按定义「红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子」，
这一处停在这里，不自己加登记表或改写引用，交主 agent 决定是否要在规格里补登记或接受这条红。

## 阶段归属表登记给 kb-scribe 的阶段

```
awk -F'\t' -v me=kb-scribe '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv | wc -l
34
```

逐个跑，原样末行与退出码：

```
10-kb-rot.sh                          ✓ kb 腐化审计通过                                                              EXIT=0
20-kb-shape.sh                        ✓ kb 形状检查通过                                                              EXIT=0
21-decision-items-sync.sh             ✓ 决策分项清单与正文同步（222 个分项）                                          EXIT=0
22-item-ref-status.sh                 ✓ 分项引用与正文状态一致（28 条决策、222 个分项）                                EXIT=0
23-link-targets.sh                    ✓ 文档指向都到得了（1228 条相对链接、37 处「第 N 节」指向）                       EXIT=0
24-status-redundancy.sh               ✓ 状态只说一遍，且分项都在对的节里（扫 237 份）                                   EXIT=0
25-kb-deictic.sh                      ✗ kb 正文里有 4 处「本轮」够不到任何日期                                        EXIT=1
26-number-name-sync.sh                ✓ 编号与简称一致（扫 224 个文件，prompts/ 按证据链原样保留）                      EXIT=0
27-format-constants.sh                ✓ 格式常量同步（19 个已登记，19 个在源码里被钉住）                                EXIT=0
28-cross-decision-status.sh           ✓ 没有把已定的决策说成未定（扫 28 条决策）                                       EXIT=0
29-settled-item-self-open.sh          ✓ 已定项的正文没有把已经定了的东西说成未定（扫 28 条决策）                        EXIT=0
30-decision-history.sh                ✓ 决策正文改了 156 行，变更史新增 6 条条目                                      EXIT=0
31-blocking-verdict.sh                ✓ 2 条未定项都判过改不改第一个事务的字节                                        EXIT=0
32-first-txn-fields.sh                ✓ 第一个事务的字段表指向都成立（查了 356 处引用）                                 EXIT=0
32-history-ordinal.sh                 ✓ 本次新增的历史条目没有撞号（存量 9 处撞号不在本阶段射程，见文件头）              EXIT=0
33-mutation-tables.sh                 ✓ 141 个实验二进制都有成形的变异表，1475 条变异的原文各命中源码一次              EXIT=0
35-user-verdict-owed.sh               ✓ 待用户复核的条款都有未还的账盯着（共 0 处）                                    EXIT=0
36-invariant-count-cross-file.sh      ✓ 不变量条数跨文件一致：实际在用 69 条，扫了 195 份 kb 文件、命中 1 处声明        EXIT=0
37-decision-summary-width.sh          ✓ 决策索引结论列都不超 200 字（共 28 行）                                       EXIT=0
38-field-table-projection.sh          ✓ 被投影的分项，字段表都投影全了（3 张字段表、24 行；投影表引用了 102 条分项）      EXIT=0
39-field-table-sum.sh                 ✓ 字段表加出来的数都对得上（表后合计 0 张，format-const 标记 2 个），另有 3 张表后 3 行内没写「合计 N 字节」、本阶段没验它们  EXIT=0
42-first-txn-trio.sh                  ✓ 第一个事务的三份文件互相挂钩（判「是」的未定项 0 条、字节表 11 节、里程碑 8 步）   EXIT=0
43-owed-table-shape.sh                ✗ 欠账表有 1 行的格数与所在表的表头对不上、0 个编号登记了两处：第 403 行 C374：7 格，而它所在那张表（「已还清」下、表头在第 371 行）是 4 格   EXIT=1
44-settled-ref-says-open.sh           ✓ 扫了 349 个文件、7438 处已定项引用，没有一处紧跟着说它没定                      EXIT=0
48-history-month-file.sh              ✓ 查了 3 份决策变更史、429 条条目，都住在日期对得上的那一份；29 份决策正文的文末没写条目  EXIT=0
49-history-brief.sh                   ✓ 决策变更史的快查与原文同步：查了 429 条条目、918 格快查，decisions-history.md 与原文一致  EXIT=0
51-admission-terms-covered.sh         ✓ 准入不等式 9 项逐项有归属：统计量 5 项，写明的例外 4 项；式子副本 4 处与权威一致   EXIT=0
52-segment-registry.sh                ✓ 比对了 5 处登记（表格 4 行 + 整条流提示 1 处……），跳过 0 条标预想的表格行         EXIT=0
53-format-const-placeholders.sh       ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位）                              EXIT=0
60-stale-open-items.sh                ✓ 没有未定项被别处的更新甩在后面                                                EXIT=0
61-settled-same-file.sh               ✓ 本次 diff 没有新增「已定」小节，本阶段无对象可判                               EXIT=0
70-citations.sh                       ✓ 全部命中                                                                    EXIT=0
67-milestone-closeout-owed.sh         ✓ 里程碑收口表收全了文件里点名、还开着的欠账号；没判 1 份（没有收口表标记）：01-first-txn.md   EXIT=0
69-evidence-in-repo.sh                ✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物   EXIT=1
```

三处红的归属判定：

- **`25-kb-deictic.sh`（是这一轮写出来的）**：命中的 4 行正是本轮插入的收口表第 37–40 行（`.claude/kb/milestone/02-second-txn.md:354–357`），
  「本轮」这个词全部来自规格第二节代码块（row 37「挡本轮，改代码」、row 38「最小改法挡本轮」、row 39「不挡本轮，交用户定」、
  row 40「不挡本轮课题」「本轮收尾时列为优先交用户定」），规格明文要求这四行「整行照抄」。按「红在这一轮写的句子上，
  停下交主 agent，不自己改句子」，这里不自行把「本轮」换成日期锚，原样报告。
- **`43-owed-table-shape.sh`（不是这一轮的）**：打中的是 `.claude/kb/checks-owed.md` 第 403 行的 C374（在「已还清」表下但仍是
  7 列），本轮只在第 354 行之后插入了 C381 一行（四列不涉及），第 403 行不在本轮改动范围内。不修，照写。
- **`69-evidence-in-repo.sh`（不是这一轮的）**：打中的是 `research/e7-index-bench/src/bin/e152_file_system_benchmark.rs`
  与 `research/mutations/e152_file_system_benchmark.tsv`（改动时间 2026-09-17），与 E152 的留存产物（2026-09-17）不对应，
  这两个文件本轮都没有碰过。不修，照写。

## 改过的全部文件：开工时与收尾时 sha256

```
                                                    开工时                                                              收尾时
.claude/kb/milestone/02-second-txn.md              f7d71731a24d1221aac770608ed19874de5c45500e7b09b27bd14b4d07eecd77      519ff54d8f3002cec4969b4fb38afa5bef93d397f4b22e04d682ec75b1d04daf
.claude/kb/checks-owed.md                          c97c28428085a99a6bd698fff4801b723051b8b29234cac2340c6cf28e012287      cc181b6f770e96455e42b128b02532a83d7affab33f6433a07dfb2d97313f2e5
.claude/kb/decisions/08-核心索引结构.md              a1b478fbe610cef65629df47bad16a66ef9af5e073b3f89740cf057a811a7271      a5f13aedcdc1b55f5e904c7fb8023a53c96f59a5cb9d5ef2b7b71c5fdddb011c
.claude/kb/layout/01-first-txn.md                  f912d445a69c5a5e8fa76b7a4967ea652d9c4bedaa84f30521d17137ddfc9038      b8db0d4613177afa506dafc7d88cff8c65387c99b755523655feaf8acf352364
```

未改动（仅记录、用于核对「不记决策变更史」）：

```
.claude/kb/decisions-history/2026-09.md            e517045b88a26395e400dd5fcee1087e5b3b2162b6780ceda6de52a36b3521ac（开工与收尾一致）
.claude/kb/decisions.md                            e56c93f96a4aea8ca2f6b01fef6e795c5c83a6bf4f492e6564d569a3f2b70a31（开工与收尾一致）
.claude/kb/decisions-history.md                    a70d60ec9732a93866e4feb59dcc92a6b41bc369ead0163950f9d2a63a0a0c16（开工与收尾一致）
```

## 没做什么

- 没判定案对不对：五处收口表改动、一行欠账表、两行决策正文、字节表一行都按规格原样照抄，未核对判决/用户定案本身是否成立。
- 没写规格外的句子：11 处替换全部逐字取自规格，未做任何概括或补写；spec.json 的构建脚本 `build_spec.py` 只是转写规格文本，未新增措辞。
- 没记决策变更史：依规格头「不记决策变更史（决策定义都没变）」，本轮未写 `.claude/kb/decisions-history/2026-09.md`，也没跑 49 号 `--write`（不在规格第五节点名的六项门禁里，也未被派发要求）。
- 没跑 `relabel-item.py`：规格未给分项翻状态指令（见上）。
- 没修红：`doc-lint.sh` 的 Z1/Z2 登记位缺口与 C381 裸引用、`25-kb-deictic.sh` 的「本轮」四处——都判定为这一轮写出、且是规格要求的逐字内容，按定义停下交主 agent 改规格，未自行改动这些句子。
- 没修不是这一轮的红：`43-owed-table-shape.sh`（C374 行形状）、`69-evidence-in-repo.sh`（E152 装置/变异表与产物时间戳不一致）——均判定不在本轮改动范围内，照写不修。
- 没提交：未做任何 git 写操作。

## 关键文件路径

- 规格：`research/prompts/m2-wave2-kb-writeback-spec.md`
- 替换规格与构建脚本：`/tmp/claude-1000/kb-m2-wave2-writeback/spec.json`、`/tmp/claude-1000/kb-m2-wave2-writeback/build_spec.py`
- 改动文件：`.claude/kb/milestone/02-second-txn.md`、`.claude/kb/checks-owed.md`、`.claude/kb/decisions/08-核心索引结构.md`、`.claude/kb/layout/01-first-txn.md`

## 追加：规格第六节更正（主 agent 2026-09-18 消息）

主 agent 判定 doc-lint 与 25 号那几处红是规格写错了（把内部格名 Z1/Z2/Z6/Z3-a、「本轮」与没带简称的 C381 裸引用写进了 kb），
更正写进 `research/prompts/m2-wave2-kb-writeback-spec.md` 第六节，18 条旧串 → 新串（16 条在 `.claude/kb/milestone/02-second-txn.md`，
2 条在 `.claude/kb/checks-owed.md`），全部落在本轮已写入的行里。43 号、69 号两处按主 agent 指示判定不是这一轮的，未重跑、未动。

### 执行

写范围闸复核（两个目标文件，均 exit=0）；旧串逐条先用 `grep -F -o … | wc -l` 核对命中数，18 条全部恰好 1 次。
规格文件：`/tmp/claude-1000/kb-m2-wave2-writeback/spec2.json`（由同目录 `build_spec2.py` 生成）。

`--dry-run` 原样输出：
```
✓ 18 处都命中得对（2 个文件），--dry-run 没写
```
实写原样输出：
```
✓ 18 处替换，写了 2 个文件，回读一致
```

回读核对：18 处替换后的原文逐一贴出核对，Z1/Z2/Z3-a/Z6 系列改成「代码三方第二轮攻方」或「正推腿」，
「本轮」系列改成「第二波代码改动」/「第二波」，`C381` 裸引用补了简称，`Z1-b、Z1-c、Z1-f` 与 `Z2-c` 两处裸编号引用换成了指向报告文件的路径（第二节 / 第三节）。

### 重跑四项门禁

```
### doc-lint.sh
  ✓ 文档铁律检查通过（检查 433，跳过 0；DOC_LINT_VERBOSE=1 看全部）
EXIT=0

### 25-kb-deictic.sh
  ✗ kb 正文里有 1 处「本轮」够不到任何日期
     .claude/kb/milestone/02-second-txn.md:355	| 38 | 实例表只增不减、实现只有一片……
EXIT=1

### 26-number-name-sync.sh
  ✓ 编号与简称一致（扫 224 个文件，prompts/ 按证据链原样保留）
EXIT=0

### 67-milestone-closeout-owed.sh
  ✓ 里程碑收口表收全了文件里点名、还开着的欠账号（判了 1 份；02-second-txn.md：全文点名 55 个 C 编号，其中还开着 50 个：表里 43 个、豁免 7 个）
    没判 1 份（没有收口表标记）：01-first-txn.md
EXIT=0
```

doc-lint、26、67 三项转绿。**25 号仍有 1 处红，规格第六节的 18 条更正没有覆盖它**：
`.claude/kb/milestone/02-second-txn.md:355`（本轮插入的第 38 行）里的「第二片或删行不在**这一轮**」——
25 号把「本轮 / 这一轮 / 上一轮 / 前一轮 / 上轮」当同一类判（脚本第 35 行的正则 `/本轮|这一轮|上一轮|前一轮|上轮/`），
这句「这一轮」是规格第二节原文里第 38 行的一部分（`…不够返回挂载错误、一个写都不发（实现员在改，被攻过零轮）；第二片或删行不在这一轮，第 370 次可写挂载之后池只能只读…`），
第六节的更正清单没有点这一处。按定义「红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子」，这里同样不自行改写，
交主 agent 判断是否要再给一条更正（例如把「这一轮」换成「第 370 次可写挂载之后」这类可锚定的说法）。

43 号、69 号未重跑（主 agent 指示不是这一轮的、照旧不动）。

### 改动文件哈希（第二轮更正前 → 更正后）

```
.claude/kb/milestone/02-second-txn.md   519ff54d…d04daf → 85357fd4…0756f82a
.claude/kb/checks-owed.md               cc181b6f…313f2e5 → d9efc58b…3238590
```
