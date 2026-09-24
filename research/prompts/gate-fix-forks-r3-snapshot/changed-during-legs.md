
.claude/rules/implementation-workflow.md：主 agent 在 2026-09-24 02:20 JST 改了 1 处（云端正推已交回、云端攻方与本地攻方还在跑，这两条腿的攻击面不含 T3）。改法：第 18 行「门禁 71 号判这一条。」换成「上游门禁的「规则纪律（项目本地）」阶段判这一条（rules-lint 扫这三处）。」（正推报告指出 71 号 2026-09-21 已删）。倒推：反着换回这一句。
.claude/scripts/gen-decision-items.py：主 agent 在 2026-09-24 02:20 JST 改了 not_number_tokens 的读法（跳过代码围栏、只认单行标记，照 doc-lint）。本地攻方按指令读快照树里的那一份，不受影响。倒推：快照树 /tmp/claude-1000/gate-fix-forks-r3-snapshot-tree/ 里就是原样。
