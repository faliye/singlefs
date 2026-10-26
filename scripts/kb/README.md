# scripts/kb

kb 与记录的定点改写、三方材料的抄录、引文与外部文献、回扫与变更史整理。

| 文件 | 做什么 |
|---|---|
| `replace-once.py`、`replace-batch.py`、`insert-row.py`、`lib_atomic_replace.py`、`lib_notation_check.py` | 共写文件时的定点替换、批量替换、插一行；`replace-once.py` 与 `insert-row.py` 写仓里的 `.md` 之前用 `lib_notation_check.py` 判新串里的钟点、时区词、时间戳（判据是 `.claude/gate.d/lib-forbidden-notations.py` 的钟点形态，与写入闸、门禁 doc-text 同一份），带的拒绝写入（退出码 7 与 6） |
| `quote-kb.py`、`kb-sections.py`、`quote-rust-items.py` | 整段抄 kb 条款（回读之外另用 `kb-sections.py` 的 `heading_bounds()` 另算 `@标题` 取法的起止行，对不上退 7；「不抄」理由点名的节出口里没有退 8；`--cited` 认路径引的 kb 文件）、生成小节清单（`heading_bounds()` 供 `quote-kb.py` import）、按项名抄 Rust 代码 |
| `kb-spec-check.py`、`relabel-item.py`、`sweep-term.py`、`stale-candidates.py`、`batch-target.py`、`stale-candidates-benchmark-facts.tsv`、`lib_basis_paths.py` | 写回规格检查（可一次给几份同批规格做跨规格状态词比对；译文编号照术语表写半角译名；「依据」里的路径不许是已跟踪文件里没进 `.claude/batch-scope` 的未提交改动，未跟踪的新文件放行并另列名单）、分项改标签、术语改名、阶段同步的候选表与它的盲写基准（`--check-facts` 同样核「出处」里的路径；`--changes` 不给 `--target` 而工作区或暂存区有相对 HEAD 的改动时拒跑，出路是 `batch-target.py`，确实整个工作区都是这一批的给 `--allow-dirty-worktree <理由>`）、把这一批的改动（点名整份取的路径与按块挑的块，块的切法同 `scripts/gate/stage-mine.py`）单独做成一个不进任何分支的提交对象当 `--target`（临时索引，真索引与 ref 不动，标准输出只打哈希）、两者共用的依据路径核法 |
| `history-consolidate.py`、`migrate-changelog-format.py`、`doc-consolidate.py`、`decision-slim-check.py`、`rules-sweep-audit.py` | 变更史与文档整理、瘦身后的丢失核对（`history-consolidate.py loss-check` 比对产物行时，反引号里的 `<路径>` 与 `git show <提交>:<同一路径>` 算同一处，路径、产物名改了别的字照旧判丢） |
| `fetch-refs.sh`、`pdf-text.py`、`verify-citations.sh` | 固定外部文献、抽 PDF 文字、核承重引文 |
| `claim-experiment.sh`、`lib_experiment_number_directories.py`、`archive-past-rounds.py` | 取实验号（查号照目录登记 `.claude/gate.d/experiment-number-directories.tsv`，读法在 `lib_experiment_number_directories.py`，门禁 doc-experiments 的 experiment-orphans 格读同一份）；早于本轮的实验记录归档（删完把引用改成 `git show <删之前的提交>:<原路径>`；可重跑，磁盘上已删而索引里还跟踪着的只报数） |
