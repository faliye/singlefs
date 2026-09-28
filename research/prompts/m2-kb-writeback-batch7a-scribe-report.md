# kb 第七批前半写回报告：实审 A3-checker-2 → I-7.13 立号

## 规格文件

- `/tmp/claude-1000/kb-batch7a-drafter/spec.json`（9 条，kb-spec-check.py 已过）
- `/tmp/claude-1000/kb-batch7a-scribe/main-agent-supplement.md`（主 agent 补充 3 条，逐字旧串/新串，未再跑 kb-spec-check.py——那是给 kb-spec-drafter 输出用的门，主 agent 直接给的补充按写回规格直接施加）
- 合并成 `/tmp/claude-1000/kb-batch7a-scribe/combined-spec.json`（12 条，顺序：spec 1-9，然后补 1/2/3；补 1、补 2 依赖补前先写入 spec 第 1 条的新文本，已在数组里排在其后，replace-batch.py 按数组顺序在内存里依次施加）

## 写范围闸预检（4 个目标文件，均 exit=0）

`.claude/kb/invariants.md`、`.claude/kb/decisions/09-加密.md`、`.claude/kb/decisions/22-单元原子性怎么合成.md`、`.claude/kb/decisions-history/2026-09.md`，检出记录 `precheck.jsonl`（主 agent 存档时草稿目录里没有这个文件，没拷进仓）。

## replace-batch.py 原样输出

- `--dry-run`：`✓ 12 处都命中得对（4 个文件），--dry-run 没写`
- 实写：`✓ 12 处替换，写了 4 个文件（改名换上新 inode），回读一致`

## 决策变更史

`.claude/gate.d/49-history-brief.sh`（不带 `--write`）先报「决策变更史的快查与原文有 1 处对不上」（生成块与按原文重新生成的不一致——是我刚写进 2026-09.md 的这一条尚未生成，不是「快查·… 还没写」缺项清单），确认没有别的轮次条目混在里面后跑 `--write`：`✓ 按原文重新生成了 .claude/kb/decisions-history.md：577 条条目，这次补了 0 条「（待补）」快查`。

`git diff -- .claude/kb/decisions-history.md` 前后核对：只改了 D9（加密）与 D22（单元原子性怎么合成）两张卡片各一行（新增本轮那条、按「只留最近 3 次」规则各挤掉最旧一条），4 处 `+`/4 处 `-`，`--stat` 显示 1 file changed, 4 insertions(+), 4 deletions(-)，没有别的轮次的行被动。

## 没有分项翻状态

这一批规格没有任何 `D<n> 分项 k` 的状态翻转（已定/未定），只改了 D9（加密） 与 D22（单元原子性怎么合成） 已定项的射程/定案文字与 invariants.md 的行，因此没有跑 `relabel-item.py`；写完正文按流程跑了 `.claude/gate.d/21-decision-items-sync.sh --write`，见下方门禁结果（✓ 350 个分项同步）。

## 依据段核对

D9（加密） 与 D22（单元原子性怎么合成） 的改动都落在「定案」/「射程」文字里（现查确认：D22 行 72、206、336 均在对应 `**依据**：` 标签行之前），没有一处改到 `**依据**：` 标签段、也没有增删 E 号引用，因此没有触发「同一份规格要带实验页那几行改动」的要求，规格也确实没有带。

## 门禁阶段（stage-owners.tsv 登记给 kb-scribe 的 41 个，逐个跑，命令与末行见 `research/prompts/kb-writeback-batch7a-tmp-evidence/gate-results.txt`（主 agent 存档时从草稿目录拷进仓））

38 绿、3 个 exit=77（本次无对象可判，不算失败：35-user-verdict-owed.sh、36-invariant-count-cross-file.sh、61-settled-same-file.sh）、3 红：

| 阶段 | 归属判断 |
|---|---|
| `52-segment-registry.sh` | 不是这一轮的：报的是 `research/scripts/replay.sh` 里 E142 产物名字与 `.claude/kb/layout/01-first-txn.md`/`checks-owed.md` 不同步；这两份 kb 文件与 `research/e7-index-bench/src/bin/e142_*.rs` 在 `git status --porcelain` 里都是单独一个 `M`/`A`（已被别的会话暂存、没有我这一轮的额外未暂存改动），不在我这批规格里，照报不修 |
| `69-evidence-in-repo.sh` | 不是这一轮的：两条都指别的会话的产物（`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`、`e157_parallel_line_one_clauses.rs` 的产物落后）与别的会话新建的 `research/prompts/_runner-compile-first-r1-*.md` 引了 `/tmp` 路径；同上核实都是单独 `M`/`A`，不在我这批规格里 |
| `97-invariant-field-anchors.sh` | **是这一轮的，但按定义不能自己补**：新写的 I-7.13 行（invariants.md:64）在正文里没有点名任何 `D<n>（简称） 已定项 k`，而这一行的文字是规格第 1 条给定的逐字新串，我不能替它补一句判据里没有的锚点句——那是「规格里没写的句子一个字不加」。**交主 agent**：需要在规格里给 I-7.13 补一句形如「D22（单元原子性怎么合成） 已定项 9 的字段」这样的锚点（已定项 9「系统配置的字段表」正是格式版本、加密类型、`physical_block_size` 等字段的定义处），或明确判断这条不变量该降级/走 C69（已定不变量没有字段可判）。 |

## doc-lint（`bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`）

末行：`✗ 文档铁律检查失败：1 个文件违规、0 处编号引用无定义、3 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 524，跳过 0）`

**红在这一轮写的句子上，停在这里交主 agent 改规格，没有自己改句子**：

1. `records/2026-09-24-里程碑二收尾调度.md:278`「正文不许写"原先 X"」——现查 `git status --porcelain` 该文件是单独 `M`（别的会话），不在我写范围也不在我这批规格里，不是这一轮的。
2. `.claude/kb/invariants.md:64` 与 `:297` 各 1 处「I-7.13 裸引用」；`.claude/kb/decisions-history.md:120`（×2）与 `:276`；`.claude/kb/decisions-history/2026-09.md:8`、`:10`、`:14`、`:16`——**都是这一轮写的**。根因：I-7.13 是这一批才新登记的编号，`kb-spec-check.py` 在起草阶段核对时它还没在 kb 里出现过，`registry` 里查不到它，因此当时那几条规则里对 I-7.13 的裸引用一律「查不到编号就跳过」（drafter 报告的 ✓ 是真的、但只覆盖了当时能查到的编号）；现在 I-7.13 写进 kb、被 doc-lint 认出登记位之后，此前规格文字里同一批的裸引用（补 1 新串「只红 I-7.13、红在…」；spec 第 9 条历史节引用记录原话「I-7.13 立号（2026-09-27）」；spec 第 8 条变更史标题、快查两行、改前改后两行都写了「I-7.13」而没跟简称）才第一次显出来。这些位置有的是**逐字引用判决/记录里的原话**（`records/2026-09-24-...md`「实审 A3-checker-2 交回并打上；I-7.13 立号（2026-09-27）」那一行），改成带简称会改动引文本身，判不判断由主 agent 定；其余（补 1 新串、spec 第 8 条标题与快查）能直接补简称，但补哪句、要不要保留原判决引文的逐字性，是内容判断，没有自己动。

## 别的会话在同一批文件上的改动

开工前 `.claude/kb/invariants.md`、`.claude/kb/decisions-history.md`、`.claude/kb/decisions-history/2026-09.md` 已经带别的会话未提交的改动（`git status` 显示 `M`），本轮全部用 `replace-batch.py` 做定点替换（先 `--dry-run` 核 12 处各恰好命中 1 次），没有整份重写，那批改动原样保留在文件里。

## sha256sum（开工时 → 收尾时）

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/invariants.md` | `eb2bcc497597f69027251032ec3f7bd484439c752705e78c1135e7ed8c18104a` | `67896c9d4ca89de93c915eb924e705cf1fc532b954854482465b0076342d3308` |
| `.claude/kb/decisions/09-加密.md` | `225b39b7892d1a00143e6fbb0ff82d11453e44fb22d7602a8c51f1896a54f703` | `5e4c7f54630f6444b5d498c71c259dc6f347aa1f3008958650b19b1bdebe68bc` |
| `.claude/kb/decisions/22-单元原子性怎么合成.md` | `79e893c3899f42e19e7dbb569a2b0755abc70a9d1f578146cd6f34f4da98696e` | `e8ae46bb065b8dce8b3874e5fc7475affb01c45f7b336eed55d7dd8ad56ac2ff` |
| `.claude/kb/decisions-history/2026-09.md` | `d69eff83dece8d790d4318431f3041b653e6d56648a6bced3ccabf6a5b0d7155` | `ce530013cc059178cefb98b938ad7832b886d9c542e231e1c70f1e4ce63ac8a4` |
| `.claude/kb/decisions.md`（未改，只按定义登记） | `0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895` | `0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895` |
| `.claude/kb/decisions-history.md`（49 号 `--write` 生成物） | `a3ed0ab086d485bccc72b2cc2a428513141a733802b426d47a22ce92e54deeb4` | `b705a66f81e7290eaf196fd2e83549ef24b0ccd1c1dd55f95d7326bfbcb5d929` |

## 没做什么

- 没有替 I-7.13 补 `D<n>（简称） 已定项 k` 锚点（97 号要的），没有给裸引用的「I-7.13」补简称（doc-lint 要的）——两处都需要对规格文字做判断或改写判决引文，交主 agent。
- 没有修 `52-segment-registry.sh`、`69-evidence-in-repo.sh`、doc-lint 里 `records/2026-09-24-...md` 那处红：三处都核实是别的会话未提交的改动，不在这批规格与写范围内。
- 没有跑 `relabel-item.py`：这批规格没有分项状态翻转。
- 没有跑重型测试（任务写明「不跑」）。
- 没有提交、没有改 `CLAUDE.md`（判决写明由主 agent 改）。
- 草稿目录 `/tmp/claude-1000/kb-batch7a-scribe/` 下都是这一轮自己写的规格、门禁记录与报告，没有编译目录、没有仓副本，无需清理。
