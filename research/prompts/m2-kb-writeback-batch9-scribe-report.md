# kb 第九批写回 —— 报告（kb-scribe）

## 规格

- `/tmp/claude-1000/kb-batch9-drafter/spec.json`（sha256 现查：`62a588f8db95a6d81ceb398d9ff769fca2254408382cf926f42482e813e02b95`，与主 agent 派发提示给的一致）
- 3 条：① `.claude/kb/decisions/23-journal的角色与格式.md` D23（journal 的角色与格式） 已定项 14 射程追加零单元发布不冻结那一句；② `.claude/kb/invariants.md` I-7.13（系统配置池级字段在读者收的范围里） 判据 / 状态列补「与 core 收同一张表」及分工现状；③ `.claude/kb/decisions-history/2026-09.md` 新增「（其二十二）」条目。三条都只改现状与依据指针，不翻分项状态、不动 D18 已定项 11 / D22 已定项 16。

## 开工前写范围闸预检（三个目标文件，逐个 exit=0，放行）

```
$ printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":".claude/kb/decisions/23-journal的角色与格式.md"}}' | AGENT_HOOK_DETECTIONS=/tmp/claude-1000/kb-batch9-scribe/precheck.jsonl bash .claude/hooks/write-guard.sh
exit=0
$ ...invariants.md... exit=0
$ ...decisions-history/2026-09.md... exit=0
```

## replace-batch.py 原样输出

```
$ python3 research/scripts/replace-batch.py /tmp/claude-1000/kb-batch9-drafter/spec.json --dry-run
✓ 3 处都命中得对（3 个文件），--dry-run 没写

$ python3 research/scripts/replace-batch.py /tmp/claude-1000/kb-batch9-drafter/spec.json
✓ 3 处替换，写了 3 个文件（改名换上新 inode），回读一致
```

三处 diff 逐字对过 spec.json 的 new 串，落点与规格一致（`git diff` 已核，见下方门禁小节的 49 号确认）。

## 决策变更史：49-history-brief.sh

`--write` 之前先跑一次不带 `--write` 的检查：报「决策变更史的快查与原文有 5 处对不上」，逐一核对，全部落在**其二十一 / 其二十 / 其十九**（2026-09-27 当天更早的条目），不是「快查还没写」形态（都不是「（待补）」，是既有内容的数字/编号对不上），也不是这一轮新写的「其二十二」。这些是别的会话已提交前留在工作区的未提交改动，不属于这一批规格。核实：写完之后再跑一次同一检查，同样 5 处、同样三个标题，「其二十二」不在列——确认我们新写的条目本身没有触发它。据此判定它们不是这一轮的（`49-history-brief.sh` 的判据不区分「还没写」与「对不上」两种形态，但都不点名我们新增的条目），照写继续，跑 `--write`：

```
$ bash .claude/gate.d/49-history-brief.sh --write
  ✓ 按原文重新生成了 .claude/kb/decisions-history.md：581 条条目，这次补了 0 条「（待补）」快查
```

补 0 条「（待补）」，印证这一轮没有缺快查的条目。`git diff` 前后对比 `decisions-history.md`：只新增了 D23（journal 的角色与格式） 已定项 14 那张卡片里我们那一行（改了什么/改前/改后），并把「共改过 75 次」改成「76 次」，按设计挤掉了卡片里最旧的一行（`D23（journal 的角色与格式） 已定项 14、D16（发布语义） 已定项 1：…系统配置见证过的更新状态读不出…`）；没有别的条目被改动。

## 三个直接编辑的文件：diff 已核对

- `.claude/kb/decisions/23-journal的角色与格式.md`：新增一行 `- ⚠️ **零单元发布这一支不冻结**：……`，紧跟在既有「切换重新读盘择根带出的三件」那条 bullet 之后，逐字与 spec.json 的 `new` 串相同。
- `.claude/kb/invariants.md`：I-7.13（系统配置池级字段在读者收的范围里） 那一行判据末尾追加「**判的字段与 core 收同一张表**：……共八项……」，状态列末尾追加「；今天 checker 只判前五项……两边补齐随代码三方第三轮那一批（判决 …Y4-a 那一格）。」，逐字与规格相同；`git diff` 里同一份文件还看到别的会话未提交的 I-9.16（树表条目按树 ID 严格升序且合发号次序） 相关改动（登记、历史节、条数 80→81 等），确认不是我们写入的，也没有被我们动过。
- `.claude/kb/decisions-history/2026-09.md`：在「## 历史版本」下插入「### 2026-09-27（其二十二）：……」条目，含两行快查、改前/改后指针与依据句，逐字与规格相同，紧接在既有「其二十一」标题之前。

## 门禁：kb-scribe 名下的阶段（`stage-owners.tsv` 登记 41 个）

```
$ awk -F'\t' -v me="kb-scribe" '$1 !~ /^#/ && NF == 3 { ... }' .claude/gate.d/stage-owners.tsv | wc -l
41
```

逐个 `nice -n 19 bash .claude/gate.d/<阶段>`：33 个 exit=0；3 个 exit=77（本次未跑：35-user-verdict-owed.sh、36-invariant-count-cross-file.sh、61-settled-same-file.sh，无对象可判）；5 个 exit=1，逐一核过点名的文件都不在这一批规格里，判定不是这一轮的，照写不修：

| 阶段 | 末行 | 点名的文件/对象 | 归属 |
|---|---|---|---|
| 10-kb-rot.sh | `✗ kb 腐化审计发现问题` | `.claude/agents/crash-verifier.md` 等 6 份定义、`.claude/rules/format-evolution.md`、`path-moves.md`、`implementation-workflow.md`、`verification.md`、两份 SKILL.md 里的失效指向 | 不是这一轮：这一批只写了 3 份 `.claude/kb/**` 文件，没碰这些定义/规则文件 |
| 33-mutation-tables.sh | `✗ 这些变异条目的「原文」在源码里不是恰好命中一次` | `research/mutations/e142_first_transaction_dry_run.tsv` 对 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` | 不是这一轮：E142 装置/变异表不在这批规格里 |
| 49-history-brief.sh | `✗ 决策变更史的快查与原文有 5 处对不上` | `decisions-history/2026-09.md` 的其二十一/二十/十九（见上一节） | 不是这一轮：与「其二十二」无关，重新核实过 |
| 52-segment-registry.sh | `✗ 段序列登记表与 E142 产物的 name=segments 行…对不上` | `.claude/kb/layout/01-first-txn.md`、E142 产物、`second_transaction_step_zero_layer0.rs` | 不是这一轮：不在这批规格里 |
| 69-evidence-in-repo.sh | `✗ 这些实验装置与变异表这一轮改了，research/results/ 里却没有一份不比它旧的产物` | `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs` | 不是这一轮：同 33 号，E142 装置改动不属于这批 |

## doc-lint.sh —— 一处红，判定属于这一轮，停下未自行处理

```
$ nice -n 19 bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .
  ✗ 编号 Y1 出现 4 次，却没有任何登记位
  ✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、1 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 527，跳过 0）
```

**这一条判定属于这一轮，我没有自行改句子，停下交主 agent 改规格。** 现查过 doc-lint 的编号扫描只扫 `*/kb/*.md`（脚本第 663 行 `kb_files`），`Y1` 在 kb 里出现 4 次：`.claude/kb/milestone/02-second-txn.md:164`、`:344`（两处都是既有内容，这一批没碰）、`.claude/kb/decisions/23-journal的角色与格式.md:407`（这一批新加的那句）、`.claude/kb/decisions-history/2026-09.md:16`（这一批新加的条目「依据」句）。阈值 `ID_MIN=3`：这一批之前 kb 里只有前两处（milestone 文件里的 2 次），没到 3 次，doc-lint 不会红；这一批新写的两处把计数推到 4，跨过阈值，触发「编号形状的记号反复出现却一处登记位都没有」（`kb-discipline.md` 第 5 条）。「Y1」「Y4-a」是判决文档 `research/prompts/m2-closeout-code-r2-main-verification.md` 第一节自己内部的格标签，不是 kb 的 D/E/C/I 编号体系，规格原文就是这么引用的（drafter 报告里也写了「判决…第一节 Y1 那一格」「…Y4-a 那一格」），我没有权限判断这个标签该不该登记、该登记成什么简称，也不该自己把这两处引用改写成别的说法——那是在替规格做判断，不是我的活。请主 agent看要不要把这两处指针改写成不撞编号形状的说法（例如把「第一节 Y1 那一格」写成不含裸「Y1」的转述），或在某份 kb 文件里给「Y1」补一个 `<!-- doc-lint:not-numbers Y1 -->` 声明（连带 Y4-a、Y2-a、Y2-b 等同一份判决里用到的其它格标签，一并考虑要不要摘）。

## 开工时与收尾时的 sha256sum

```
开工时（sha-before.txt）：
609f1a2e5d437759392cbd244910fbe7a8d61acea849de96965944f3e3007028  .claude/kb/decisions/23-journal的角色与格式.md
52d4703f50d1cf14b6208bdc6cde006d59e246b784ab7a32250d5adee54a17f6  .claude/kb/invariants.md
c02c7f0daaddf2393966b5c7f4293e462b6d74129a1095c0cd357e3098a79430  .claude/kb/decisions-history/2026-09.md
6838719218f80e6964b2a06cf94ef48d8b59779cb7f4843bfb25ed963844bc0b  .claude/kb/decisions-history.md
8d23f9881c659c2b7988fa5bd44320222f80c97769539796742e64bc524e5315  .claude/kb/decisions.md

收尾时（sha-after.txt）：
2147d06ba2b0c506dd67b38fda71fd1a975cefbad48bc64247be6f145d825c3f  .claude/kb/decisions/23-journal的角色与格式.md
78a63ecbd3b8b92764a75113804e388aecc7624857919de64bfda0ad95f5f9fe  .claude/kb/invariants.md
4d831f1deb74c9748e34aff37362575a684ac7ccc419cf2e165ae183c73a5908  .claude/kb/decisions-history/2026-09.md
35bce95757635ab335a93b3add57db83e676b7d30b24fc4a3047202c0bb46db8  .claude/kb/decisions-history.md
8d23f9881c659c2b7988fa5bd44320222f80c97769539796742e64bc524e5315  .claude/kb/decisions.md（未改，哈希相同）
```

## 全部改过的文件

- `.claude/kb/decisions/23-journal的角色与格式.md`
- `.claude/kb/invariants.md`
- `.claude/kb/decisions-history/2026-09.md`
- `.claude/kb/decisions-history.md`（由 `49-history-brief.sh --write` 重新生成）

## 没做什么（固定会有的）

- 没判定案对不对，没写规格外的句子。
- 没修 10、33、49、52、69 号的红：已核实不是这一轮的改动，照写不修。
- 没自行处理 doc-lint 的「Y1」红：判定是这一轮引入的，停下交主 agent 改规格，没有自己改句子或加 `not-numbers` 声明。
- 没跑 21-decision-items-sync.sh --write（不属于这一批：这批不翻分项状态，21 号已作为常规阶段跑过并绿，不需要额外触发写）。
- 没跑 relabel-item.py（这批不是分项翻状态）。
- 没提交。
