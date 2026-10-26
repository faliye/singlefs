# 工具层拒写的文件名前缀实测（第九批，拆分提案第九十节 ㉑）

日期：2026-10-22。方法：只用 Write 工具，在草稿目录 `~/.local/share/singlefs/work/landing-audit-b9-prefix-probe/` 里逐个建文件，内容一行「探测」；十二次调用同一轮并发发出，各自结果互不依赖。Bash 没用来建任何探测文件。

被拒的原文（七次逐字相同）：

> `<tool_use_error>Subagents should return findings as text, not write report files. Include this content in your final response instead.</tool_use_error>`

写成的结果：`File created successfully at: <路径> (file state is current in your context — no need to Read it back)`。

| 名字 | Write 结果 | 被拒时的原文 |
|---|---|---|
| `report-probe.md`（阳性对照） | 被拒 | 见上，逐字相同 |
| `probe.md`（阴性对照） | 写成 | - |
| `SUMMARY.md` | 被拒 | 见上，逐字相同 |
| `summary-x.md` | 被拒 | 见上，逐字相同 |
| `FINDINGS.md` | 被拒 | 见上，逐字相同 |
| `findings-x.md` | 被拒 | 见上，逐字相同 |
| `ANALYSIS.md` | 被拒 | 见上，逐字相同 |
| `analysis-x.md` | 被拒 | 见上，逐字相同 |
| `x-summary.md` | 写成 | - |
| `x-findings.md` | 写成 | - |
| `x-analysis.md` | 写成 | - |
| `summary.txt` | 写成 | - |

两份对照：阳性对照 `report-probe.md` 被拒，阴性对照 `probe.md` 写成，都符合预期，本轮有效。

落盘核对（`ls -A`，删探测文件之前）：目录里只有 `probe.md`、`summary.txt`、`x-analysis.md`、`x-findings.md`、`x-summary.md` 加锁文件，被拒的七个名字都没建出来。写成的五个探测文件已用 `rm` 删掉，删后目录只剩 `.owner`、`.writer-claim-82ee79e829d5609e`、`.writer-lock` 和本报告。

## 结论

`REPORT`、`SUMMARY`、`FINDINGS`、`ANALYSIS` 四个前缀的 `.md` 都被工具层拒，且大写（`SUMMARY.md`、`FINDINGS.md`、`ANALYSIS.md`）与小写（`summary-x.md`、`findings-x.md`、`analysis-x.md`、`report-probe.md`）一样被拒，「不分大小写」成立；这几个词放在名字中间或末尾（`x-summary.md` 等）不拦，只认开头；`summary.txt` 不是 `.md`，不拦。`.claude/agent-common.md` 第 108 行「以 REPORT、SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 `.md`」与实测一致。

推翻条件：换一个大小写混写（如 `Summary.md`）或别的目录再测，有一例写成就不是「不分大小写」或「不分目录」；本轮没测混写和别的目录。

## 没做什么

- 没测混合大小写（`Summary.md`）、前缀后紧跟别的字母的（`summaryx.md`）、`.markdown` 等别的扩展名、别的目录。
- 没测 Edit 工具建文件；没测 Bash 写（派发提示说不拦，没验）。
- 没跑重型测试，没提交，没改仓内文件。
