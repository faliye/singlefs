<!-- knowledge-sync -->
# c468-evidence 阶段同步

触发文件：.claude/gate.d/68-knowledge-sync.sh、.claude/gate.d/91-archive-past-rounds.sh、research/scripts/archive-past-rounds.py、research/scripts/test-environment-check.py

这一阶段做成的事：还清 C468（判据 ⑤「## 原始证据」）、新立 C470 / C472 / C473 / C474、门禁 91 号与 `archive-past-rounds.py` 的出路打出基准且 `--apply` 不给基准当场拒绝、`test-environment-check.py` 的 clean 汇总行报出「不碰几项、其中几项只是太新」并逐个点名、第三轮三方判完。

## 搜索

事实表 8 行，候选表 258 行（组 F1 / F3 / F4 / F5；F2、F6、F7、F8 是新立事项，按 `build_candidates` 不出候选）。检索词与命中数（`current_state_corpus` 口径，基准 `d6a9676` → 暂存树）：

```
F1  C468      基准 1   结束 2   候选 2 行
F3  归档      基准 53  结束 55  候选 55 行
F4  残留      基准 195 结束 197 候选 197 行
F5  逐行全看  基准 2   结束 4   候选 4 行
python3 research/scripts/stale-candidates.py --check-facts research/prompts/c468-evidence-facts.tsv --base d6a9676 --target <暂存树>
  → ✓ 事实表罩全了：0 条变更记录都有出处，8 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
python3 research/scripts/stale-candidates.py --check-report research/prompts/c468-evidence-candidates.tsv research/prompts/c468-evidence-judge-f1-f3-f5.md research/prompts/c468-evidence-judge-f4.md research/prompts/c468-evidence-judge-main-addendum.md
  → ✓ 报告判全了：258 行候选都有逐行判定
```

逐行判的结果：258 行里「不相干」254 行、「事件句不改」8 行（两数由三份报告的判定列数出来，合计 262 行判定——其中 4 行是插了三条欠账之后行号挪位、同一行内容在旧行号上的那一份，见补判报告开头），**候选行里「要改」「要人看」各 0 行**——这一阶段没有把任何一句现状话变成假话。候选表之外另有一行反向核查判「要补」（M1），处置写在「## 命中处置」那一行。

主 agent 的逐行全看：每一条判定都对着暂存树那一版的原文核过。核出两类问题，都不改判定：

1. **六行的理由抄原话不逐字**：丢了原文的 `**` 强调标记（`.claude/kb/checks-owed.md:33`、`:208`、`.claude/kb/decisions/15-格式冻结政策.md:222`、`records/2026-08-29-审计轮-外部引用复核与阻塞集.md:70`）、丢了编号的简称括注并多一个空格（`.claude/kb/experiments/07-离线索引harness.md:172`）、丢了「」（`.claude/kb/decisions/08-核心索引结构.md:247`）。六行的判定主 agent 逐行读过原文，都站得住。`--check-report` 对这一类判绿，账记在 C473（逐行判的报告不核引的原话在不在那一行）。
2. **F4 那一段报了一行 M1「要补」**：说这一阶段改的 clean 汇总行在 `.claude/kb/checks-owed.md` 里没有任何一处登记。处置见「## 命中处置」那一行。

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表 | research/prompts/c468-evidence-facts.tsv | 9 |
| 候选表 | research/prompts/c468-evidence-candidates.tsv | 259 |
| 逐行判定报告（F1 / F3 / F5 段） | research/prompts/c468-evidence-judge-f1-f3-f5.md | 76 |
| 逐行判定报告（F4 段） | research/prompts/c468-evidence-judge-f4.md | 208 |
| 逐行判定报告（主 agent 补判的 10 行） | research/prompts/c468-evidence-judge-main-addendum.md | 31 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:445 | 读它汇总行里的「判红 N 类」，N 不是 0 就红、汇总行读不到也红，出路是它自己打印的 `clean` 计划 | 不改：F4 那一段报 M1「要补」，判不补。欠账表只登记两样——欠着的检查，与已还清的欠账；这一次改的是一个已有脚本的汇总行多报一句数，它从来不是一笔欠账，登记进去等于把一个非检查塞进检查登记表。它的钉子是脚本自检里新加的两条断言（把「太新」那一档抹掉必判红，已自证），落点是这份同步记录与提交说明。C396 这一行说的是 77 号怎么读「判红 N 类」，而这次只在同一行汇总里**追加**一句，没动那句怎么写，所以这一行本身也不用改 |

## 交给上游的一条

`gate.sh` 没有「只重跑上一趟红的那几道」的模式：77 道、一趟约 20 分钟，任何一道红就要整趟重跑，而红的那道常常几秒钟跑完。2026-09-22 两趟白跑，红的四道加起来不到 10 秒。形态：`--rerun-failed` 读上一趟的阶段结果索引，只跑上一趟非绿的与读同一批输入的那几道，跑绿之后明说「这不构成全量绿」、不许前移 `refs/sop/gate-ok`；判别力自证是造「A 红 B 绿」、修好 A，它必须只跑 A 且拒绝报全绿。`gate.sh` 在上游 `singlefs-ai-sop` 里，本仓只提不改，归做发版的会话。
