## D7 是否进 Linux 主线 —— 已定

**前几年不进。按单人项目做。**

依据：bcachefs 被移出主线是社交失败不是技术失败。进主线意味着社交成本、
review 周期、和维护者关系全部要进预算，且不可控；也意味着格式提前冻结。
不进主线则格式想改就改，节奏自己定，与 [format-evolution.md](../../rules/format-evolution.md)「在第一个外部用户出现之前，磁盘格式是软的」相容。

本工程的准入判据与 Linux 完全不同，见 `.claude/singlefs-ai-sop/rules/show-me-test.md`。

## 历史版本

D7（是否进 Linux 主线）的历史条目集中在 [decisions-history.md](../decisions-history.md)。
