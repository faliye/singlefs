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
@.claude/singlefs-ai-sop/rules/test-discipline.md
@.claude/singlefs-ai-sop/rules/evidence-discipline.md
@.claude/singlefs-ai-sop/rules/verify-before-claiming.md
@.claude/singlefs-ai-sop/rules/command-safety.md
@.claude/singlefs-ai-sop/rules/writing-economy.md
@.claude/singlefs-ai-sop/rules/session-wrapup.md

## 规范从哪来

| 项 | 值 |
|---|---|
| 上游仓 | `singlefs-ai-sop`，本机在兄弟目录 `../singlefs-ai-sop-zh`。同一份规范有多语言版本，**对外以 `-en` 为准**；本项目只接其中一份，不需要知道别的 |
| 项目里的副本 | `.claude/singlefs-ai-sop/`，是**拷贝，不是符号链接**；与上游同步靠重新拷贝一份 |
| 版本戳 | `.singlefs-ai-sop-version`。门禁第一阶段拿它跟副本的 `VERSION` 比，对不上就红——那是在提醒「规矩变过了，先读再跑」 |
| 怎么改 | 共享规则只能在**上游**改并抬 `VERSION`，然后同步副本、跑 `bash .claude/singlefs-ai-sop/install.sh` 刷版本戳。**不许在 `.claude/singlefs-ai-sop/` 里就地改**——下次同步就没了 |

上游管的是「项目怎么和 AI 协作」，不管文件系统怎么设计；
只有本工程需要的纪律放 `.claude/rules/`，不要往上游推。

## 项目本地规则

@.claude/rules/fs-design.md
@.claude/rules/format-evolution.md
@.claude/rules/three-way-inference.md

文件系统的设计纪律只有本工程需要，所以它不在共享 SOP 里——
共享 SOP 管的是「项目怎么和 AI 协作」，不管某一类系统怎么设计。

（上面那批是 [singlefs-ai-sop](.claude/singlefs-ai-sop/README.md) 分发的共享规则，
**改它们等于改所有项目**——要改就改上游并抬 `VERSION`，不许在项目里就地改。
上游的改动应当罕见：经常变说明规范本身没设计好。**作业在本仓，不在上游仓。**）

## 项目本地事实

| 文件 | 内容 |
|---|---|
| `.claude/kb/decisions.md` | 设计决策：定了什么、为什么、还没定什么 |
| `.claude/kb/invariants.md` | 不变量清单，checker 是它的可执行形式 |
| `.claude/kb/prior-art.md` | 他家方案调研，含来源与口径 |
| `.claude/kb/pitfalls.md` | 避坑清单，每做设计决定回来对一遍 |
| `.claude/kb/checks-owed.md` | 欠的检查：知道要拦什么但还拦不了的，含前置 |
| `.claude/rules/` | 项目本地规则（`fs-design.md` 设计纪律、`format-evolution.md` 格式演进纪律、`three-way-inference.md` 推论三方论证） |
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
bash .claude/gate.d/10-kb-rot.sh      # kb 腐化：引用悬空、结论悬空、条数对不上
bash .claude/gate.d/20-kb-shape.sh    # kb 形状：用词、指代、链接、条数与标题相符
```

**`.claude/gate.d/*.sh` 是项目本地门禁阶段**，`gate.sh` 按文件名顺序逐个跑，
每个记成一个独立阶段。放进去的检查**会红**，不是提醒句。
新增一条：写个 `.sh` 丢进去，头部写 `# gate-stage: <阶段名>`。
⚠️ **脚本存在但跑不起来（没执行位、语法错）一律判红，不许当成跳过。**

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
