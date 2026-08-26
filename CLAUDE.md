# singlefs

**一个从零设计的 COW 文件系统，Rust 实现。**
现有 COW 文件系统是**设计输入**（它们的病历和解法），不是移植目标。

当前里程碑：**格式设计阶段**——尚无磁盘格式、无代码。
先定 `.claude/kb/decisions.md` 里的待定项，再动第一行实现。

## 规则（始终生效）

@.claude/singlefs-ai-sop/rules/show-me-test.md
@.claude/singlefs-ai-sop/rules/machine-first.md
@.claude/singlefs-ai-sop/rules/doc-discipline.md
@.claude/singlefs-ai-sop/rules/design-doc-discipline.md
@.claude/singlefs-ai-sop/rules/kb-discipline.md
@.claude/singlefs-ai-sop/rules/fs-design.md
@.claude/singlefs-ai-sop/rules/format-evolution.md
@.claude/singlefs-ai-sop/rules/test-discipline.md
@.claude/singlefs-ai-sop/rules/evidence-discipline.md
@.claude/singlefs-ai-sop/rules/verify-before-claiming.md
@.claude/singlefs-ai-sop/rules/command-safety.md
@.claude/singlefs-ai-sop/rules/writing-economy.md
@.claude/singlefs-ai-sop/rules/session-wrapup.md

（这些是 [singlefs-ai-sop](.claude/singlefs-ai-sop/README.md) 分发的共享规则，
**改它们等于改所有项目**——要改就改上游并抬 `VERSION`，不许在项目里就地改。）

## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | 设计决策：定了什么、为什么、还没定什么 |
| `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `records/` | 建设过程 |

## 门禁

门禁的目的是**把每一份提交抬到值得花人的时间去看那条线上**，不是把谁挡在外面。
它不按来源区分提交者，只区分带证据的和不带的。

```bash
bash .claude/scripts/gate.sh          # 准入门禁，提交前必跑
GATE_QEMU=1 bash .claude/scripts/gate.sh   # 再加 QEMU harness 自检

bash .claude/scripts/check.sh         # 快速反馈（格式/lint/构建/单测）
bash .claude/scripts/lkmm.sh          # 内存序（herd7 + litmus/）
bash .claude/scripts/qemu.sh --selftest    # QEMU harness 自检
bash .claude/scripts/gate-lint.sh     # 门禁自身：每条拒绝是否都给了下一步
bash .claude/scripts/env.sh           # 环境自检
```

**Gate proves evidence requirements, not semantic correctness.**
绿色只说明证据要求被满足，不代表语义正确——`gate.sh` 每次都会列出未实现的阶段。

## 本项目的特殊性

1. **没有 oracle。** 从零设计意味着没有参照实现可比对——移植类项目那种
   「拿现成工具的输出当标准答案」的便利这里不存在。功能正确性只能靠模型对拍，
   这是最大的隐性成本，见 `kb/prior-art.md` 第三节。
2. **格式还是软的。** 第一个外部用户出现前可以随时拆了重做。
   真正要慎重的是 `kb/decisions.md`，不是 `.rs` 文件。
3. **不进 Linux 主线**（D7）。前几年按单人项目做，准入判据是门禁——
   每个 patch 都要经过严格测试，不按提交来源区别对待。

## 一句话版本

- 先定决策，再写代码——D4/D8 未定之前写下去的实现多半要返工。
- 从事务开始，不从功能开始；第一个可运行目标是「正确提交一个事务」。
- 记账必须在提交时增量维护；任何要「事后扫一遍」的记账设计当场否决。
- 门禁全绿**不构成崩溃一致性证据**——崩溃点重放还没接进来。
