<!-- 2026-09-28 从会话 d16a74c5 的记录（2026-09-21T17:27:09Z 那次 Bash heredoc）逐字抽回；原件 /tmp/claude-1000/continue-past-errors/spec.md 已不存在 -->

# 改法：注入的块设备错之后，让历史继续往下跑

判决 `research/prompts/m2-supp3-item4-code-r1-main-verification.md` 判定一打中的那一格。用户 2026-09-21 定「与补 checker 两条并行」。

## 今天是什么样

`crates/singlefs-harness/src/history.rs`（`execute_history_with_faults` 那一段，攻方探针打在 `@@ -2950,9` 一带，行号自己现查）：注入的块设备错让模型报「该成却拒了」，执行器当场 `return Some(observation)`，**这一整段历史剩下的步一步都不跑**。

产物对照（同样 24 段、同样 95 个注入点，只有停不停这一件事不同）：

```
research/prompts/m2-supp3-item4-code-r1-opus-model/baseline-fast.txt:114
  走到模型认下的某一版 77 次、走到失败那一步正在写的那一版 18 次（收口表第 40 行那一格，只记不判）、模型都不允许的 0 次
research/prompts/m2-supp3-item4-code-r1-opus-model/baseline-fast.txt:116
以「已知红」收尾 {0: 3, 1: 4}、新发现 0

research/prompts/m2-supp3-item4-code-r1-opus-model/probe-k2-continue.txt:71
  走到模型认下的某一版 74 次、走到失败那一步正在写的那一版 7 次（收口表第 40 行那一格，只记不判）、模型都不允许的 14 次
research/prompts/m2-supp3-item4-code-r1-opus-model/probe-k2-continue.txt:139
以「已知红」收尾 {0: 5, 1: 4}、新发现 38
```

攻方的探针在 `m2-supp3-item4-code-r1-opus-model/probes.patch` 里，**只读它当参考，不许照搬**：那是靠环境变量开的临时开关，这次要做成正式规则。

## 放行条件

攻方探针用的五个合取条件（`probes.patch` 里逐字）：没有违例、没有 panic、没有 harness 判定、模型分歧恰是 `RefusedWhenModelRequiresSuccess`、结局恰是 `Refused { member }` 且 member 含 `BlockDeviceError`。

**这五条当起点，自己核一遍每一条今天还在不在**（`history.rs` 与 `fault_injection.rs` 这一轮刚被改过，`ModelDisagreementAspect` 的成员名、`StepOutcome::Refused` 的形状都要现查）。核完在报告里逐条写「还在 / 改名成什么 / 不存在了」。

## 归因口径：先全报，不预先豁免

放行之后，后续步骤里冒出来的违例怎么算，是这条改法唯一真正要定的事。**这一轮的口径是不豁免**：

- 后续步骤的违例**全部报成新发现**，一条都不预先归到「注入的必然后果」里
- 每条新发现带上**两个步号**：注入落在第几步、违例在第几步，以及两者的距离
- 快档的收尾行照旧报「以『已知红』收尾 {...}、新发现 N」

**判据**：判定一打中的病就是「用规则把结果做漂亮」，先定一套豁免规则等于把同一个病换个地方再犯一次。先把 38 条的形状摆出来，看清哪些真是注入的直接投影、哪些是实现的缺口，再定豁免——那是下一轮的事。

⚠️ 这与同一轮里「说谎的设备之后盘面不一致」那一格的白名单**不冲突**：那一格豁免的是设备说谎这一类故障的必然后果（写报成功而什么都没写，没有实现防得住），而且白名单里只有两个签名、名单外照报。这一条放行的是**设备如实报错之后**的后续步骤——实现收到了错误、该自己收拾干净，收拾不干净就是缺口。

## 验收

- 快档重跑，收尾行的「新发现」从 0 变成正数，每条带注入步号、违例步号与距离
- 「模型都不允许的 N 次」那一格从 0 变成正数
- `注入之后 core panic 的次数` 仍为 0
- 逐条把新发现按「违例签名」聚合报一张表（签名 → 次数 → 第一个复现种子），**交主 agent 判哪些是缺口**
- 新发现里凡是能单种子复现的，各给一个写死种子的用例；复现不了的照实写「这条今天单独复现不出来」

## 变异

`crates/mutations.tsv` 末尾追加至少两条：

1. 把放行条件整个去掉（改回当场 `return`），点名的用例必须红
2. 把「后续违例全报」改成「一律不报」，点名的用例必须红

## 不做的

- 不定豁免规则（上面写了为什么）
- 不碰 `crates/singlefs-checker/`（另一条线在改）
- 大档不跑
- 不提交、不 `git add`、不写 kb
