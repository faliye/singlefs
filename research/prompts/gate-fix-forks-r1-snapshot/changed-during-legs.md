# 腿跑着的时候被别的会话改过的快照内文件（倒推办法与处数）
.claude/gate.d/47-research-script-selftests.sh：singlefs-ca 会话在 cache-keepalive.sh 那一行之后定点插入 1 行 `"bash research/scripts/watch.sh --selftest" \`（它 2026-09-23 下午告知）。倒推：删掉这 1 行，副本存为 47-research-script-selftests.sh.at-snapshot。
