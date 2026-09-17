# alloc-basis-r2 本地辩方腿：运行记录（2026-09-17）

提示：`research/prompts/alloc-basis-r2-local-defense.md`。核对表：`research/prompts/alloc-basis-r2-local-defense-translation-audit.md`。前台跑 `research/scripts/ask-local.sh`，不 `setsid`、不 `&`、不 `disown`。

要辩护的一方：Z3 的「串」读法（`research/prompts/_alloc-basis-r2-body.md:69`）。攻方问题（主 agent 派发提示原文，整行抄）：「主 agent 的倾向是『合』，理由是串读法下『已释放而不可再分配』的空间在第一道闸上不算可用，只有抬 F 才装得下的请求在第一道闸就被拒、D16 的抬 F 路径走不到。」

| 样本 | 退出码 | 词数 | oov-check.py 生词 | 拼接 | 判定 |
|---|---|---|---|---|---|
| s1 | 0 | 665 | subtracts、attempt's（生词=2，均为合法英文词，非损坏） | 0 | 干净 |
| s2 | 0 | 596 | subtracts、insufficiency、computable（生词=4，工具报的数与列出的词数不一致，逐词核对均为合法英文词，非损坏） | 0 | 干净 |

两份样本一次尝试即干净，没有作废副本（没有 `-output-void*.md`）。两次尝试都在两次之内拿到，未触发「连续五次拿不到两份干净就停」的停止线。

本地网关（`~/code/ai-center` :8200）全程可用，没有缺席。

样本逐问差异（如实记录，不解读、不总结、不采纳）：

- 1a：两份样本都把攻击拆成「gate 1 无条件扣 defer_pending_release」（各自引 F2/F3）与「gate 1 先于 gate 2、失败即拒」（各自引 F1 与 SERIAL 定义句）两部分，逐词表述不同但引用的事实编号一致。
- 1b：两份样本都判定「df 等于 F2 的完整可用量」，都引 F5 的 df 句与 F6 第 1 条，链条一致；s1 额外提到探针实测的 24 次假性 ENOSPC 数字作为佐证，s2 没有引这个数字。两份都没有按 F5 自己的提醒（「这是开放的解读问题」）明确说「我选的是哪种读法、为什么」，而是直接断言「equals」，跳过了提示里要求的「state explicitly which reading you rely on」这一步。
- 1c：两份样本给出的机制形状相同：另一个先过 gate 1 的准入请求触发 gate 2 的抬 F，抬 F 调用 F8 的 reclaim 机制，缩小 defer_pending_release，从而抬高 gate 1 的可用量，惠及原本卡住的那个请求。两份都提到「不等于串-重判」（s1 一句带过，s2 没有专门回应这条守卫句，但机制描述本身没有假设同一次准入内部重判 gate 1）。s2 额外给了一句关于「3 次发布的界」的补充（"filesystem pushes for raising F do not count toward the bound"），s1 没有这句。
- 1d：两份样本结论一致：E139 的越界说明「即便是项目实际采用的机制（D16 已定项 1），字面上的界 3 也没有总是达到」，据此判定「串」在这一点上不比已采用的机制更差。两份的措辞非常接近。
- 1e：两份样本都判定 DOES NOT HOLD，且都把 F6 的 ⚠️ 警告（E139 越界那句）当成「挡住脱身」的那个事实——这与 1c 里刚给出的机制之间的关系，两份样本都没有解释清楚（1c 说机制成立，1e 又说 DOES NOT HOLD，两步之间是否矛盾，样本自己没有讲清楚），如实记录，不代主 agent 判断。

没做什么：不解读、不总结、不采纳本地模型的答复；两份样本 1c 与 1e 之间是否自相矛盾，不由本报告下结论，留给主 agent 与判决。
