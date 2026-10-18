# scripts/kb

kb 与记录的定点改写、三方材料的抄录、引文与外部文献、回扫与变更史整理。

| 文件 | 做什么 |
|---|---|
| `replace-once.py`、`replace-batch.py`、`insert-row.py`、`lib_atomic_replace.py`、`lib_notation_check.py` | 共写文件时的定点替换、批量替换、插一行；`replace-once.py` 与 `insert-row.py` 写仓里的 `.md` 之前用 `lib_notation_check.py` 判新串里的钟点、时区词、时间戳（判据是 `.claude/gate.d/lib-forbidden-notations.py` 的钟点形态，与写入闸、门禁 doc-text 同一份），带的拒绝写入（退出码 7 与 6） |
| `quote-kb.py`、`kb-sections.py`、`quote-rust-items.py` | 整段抄 kb 条款（回读之外另用 `kb-sections.py` 的 `heading_bounds()` 另算 `@标题` 取法的起止行，对不上退 7；「不抄」理由点名的节出口里没有退 8；`--cited` 认路径引的 kb 文件）、生成小节清单（`heading_bounds()` 供 `quote-kb.py` import）、按项名抄 Rust 代码 |
| `kb-spec-check.py`、`relabel-item.py`、`sweep-term.py`、`stale-candidates.py`、`stale-candidates-benchmark-facts.tsv` | 写回规格检查、分项改标签、术语改名、阶段同步的候选表与它的盲写基准 |
| `history-consolidate.py`、`migrate-changelog-format.py`、`doc-consolidate.py`、`decision-slim-check.py`、`rules-sweep-audit.py` | 变更史与文档整理、瘦身后的丢失核对 |
| `fetch-refs.sh`、`pdf-text.py`、`verify-citations.sh` | 固定外部文献、抽 PDF 文字、核承重引文 |
| `claim-experiment.sh`、`archive-past-rounds.py` | 取实验号；早于本轮的实验记录归档 |
