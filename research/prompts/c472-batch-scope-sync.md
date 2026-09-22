<!-- knowledge-sync -->
# c472-batch-scope 阶段同步

触发文件：.claude/gate.d/11-batch-scope.sh、.claude/gate.d/68-knowledge-sync.sh、.claude/gate.d/knowledge-sync-triggers.tsv、.claude/gate.d/stage-owners.tsv

这一阶段做成的事：还清 C472（新立门禁 11 号，核这一批的触发文件都登记进 `.claude/batch-scope`）、11 号的基准取 HEAD 不取 `GATE_BASE`、触发文件的清单从 68 号的内嵌 python 挪进 `.claude/gate.d/knowledge-sync-triggers.tsv`（两道读同一份，11 号第六条钉住 68 号还在读它）、11 号在 `.claude/gate.d/stage-owners.tsv` 里登记归 `gate-triage`。

## 搜索

事实表 4 行，一件做成的事一行；候选表 17 行。检索词与命中数（`current_state_corpus` 口径，基准 `d037e0a` → 暂存树）：

```
F1  C472          基准 1   结束 1   候选 1 行
F2  触发文件       基准 3   结束 3   候选 3 行
F3  GATE_BASE     基准 3   结束 4   候选 4 行
F4  stage-owners  基准 9   结束 9   候选 9 行
python3 research/scripts/stale-candidates.py --check-facts research/prompts/c472-batch-scope-facts.tsv --base d037e0a --target <暂存树>
  → ✓ 事实表罩全了：0 条变更记录都有出处，4 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）
python3 research/scripts/stale-candidates.py --check-report research/prompts/c472-batch-scope-candidates.tsv research/prompts/c472-batch-scope-judge-all.md
  → ✓ 报告判全了：17 行候选都有逐行判定
```

**检索词返工过一次**：第一版四件事压成一行、检索词只写 `C472`（1 行命中），候选表 1 行。按「检索词写新旧说法都会提到的概念名词、宁宽勿窄」拆成四行、换成上面四个词之后涨到 17 行。多出来的 16 行里有一行是真要判的（`records/2026-09-16-subagent拆分提案.md:308`，见「## 命中处置」）——窄检索词下它一辈子进不了候选表。形态是 C470（事实表标新立的行没有独立证据） 那一格的第二半。

主 agent 的逐行全看：17 行逐条对着暂存树那一版的原文核过，**两处改判**，写在 `research/prompts/c472-batch-scope-judge-main-overturn.md` 里。机械那一半另跑一段脚本（C473（逐行判的报告不核引的原话在不在那一行） 的原型）逐行比对「引的原话在不在那一行上」，17 行全过。

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表 | research/prompts/c472-batch-scope-facts.tsv | 5 |
| 候选表 | research/prompts/c472-batch-scope-candidates.tsv | 18 |
| 逐行判定报告 | research/prompts/c472-batch-scope-judge-all.md | 52 |
| 主 agent 逐行全看与两处改判 | research/prompts/c472-batch-scope-judge-main-overturn.md | 39 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| .claude/kb/checks-owed.md:397 | 判别力自证用一个「上次门禁绿之后别人提交过一次」的样本仓，必须红 | 改了：这一行原判「不相干」，主 agent 逐行全看时改判「要改」。C449 要的那道检查逐字是「两者不同就判红」，而门禁 11 号取 HEAD 是有意的（它问的是「这一次提交要带哪些」）⇒ 照它现在写的形态建出来，11 号会被冤枉。在「怎么拦会红」一栏补了一句：有意取别的基准的阶段要另列一张显式豁免表，写明它取的是什么基准、问的是哪个别的问题，表里没有的才比对；点名 11 号与 68 号判据 ⑤ 是这一类 |
| records/2026-09-16-subagent拆分提案.md:308 | 每个项目本地阶段先由哪个 agent 跑，登记成一张表，只写这一份 | 不改：这一行原判「要改」，主 agent 改判「事件句不改」。那七个数确实全部过时（主 agent 独立数过：`kb-scribe` 37、`gate-triage` 15、`experiment-runner` 13、`crash-verifier` 7、`implementation-writer` 6、`three-way-materials` 2、`sweep` 2、`prior-art` 1、`mutation-triage` 1），但它住在带日期的小节 `## 十五、…（2026-09-17）` 下、列头逐字是「做了什么」，主语动词是「登记成一张表」。判据是「把这句话里的旧值换成新值，它还是不是真话」——换成新数之后它对 2026-09-17 就成了假话 ⇒ 说的是那一次，留着 |
| .claude/kb/checks-owed.md:436 | 理由里至少有一段 4 个字以上带引号的原话在那一行里 | 改了：C473 那一条的「怎么拦会红」补了两种合法写法——反引号里的片段也算引原话、用省略号标出的省略按段核（每段至少 4 个字、逐段都要找得到）。依据是这一轮的实测：原型第一版两样都不认，17 行里误判 3 行；认下来之后 0 行，同一份报告一个字没改 |

## 这一批与上一批的对照

同样是 4 个触发文件，上一批（`c468-evidence`）候选表 258 行、逐行判下来 254 行「不相干」，因为里面有两个是「顺手修」；这一批候选表 17 行、判出两处真要处置的。差别不在触发文件的个数，在它们是不是这一批真要的——这正是门禁 11 号要人在开工时先写一行字的理由。
