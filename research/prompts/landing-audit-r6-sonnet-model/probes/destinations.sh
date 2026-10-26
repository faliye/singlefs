#!/bin/bash
# 用法：destinations.sh <仓根> <模型目录>
# 对每一项的关键字符串，数它在几处「去向」里出现：九十节（输入快照 inputs/proposal-section-90.txt）、调度记录（inputs/dispatch-record.md）、
# r4 判决、r5 判决、r6 正文（这三份在仓里不再改）、欠账表（仓里现读）。0 表示这一处没有这个字符串；有命中的要人去看是去向还是出处。
R="$1"; M="$2"
count() { if [ -f "$1" ]; then grep -cF -- "$2" "$1"; else echo NA; fi; }
row() {
  id="$1"; s="$2"
  printf '%-4s | %-30s | 九十节 %s | 记录全文 %s | r4判决 %s | r5判决 %s | r6正文 %s | 欠账表 %s\n' "$id" "$s" \
    "$(count "$M/inputs/proposal-section-90.txt" "$s")" "$(count "$M/inputs/dispatch-record.md" "$s")" \
    "$(count "$R/research/prompts/landing-audit-r4-main-verification.md" "$s")" "$(count "$R/research/prompts/landing-audit-r5-main-verification.md" "$s")" \
    "$(count "$R/research/prompts/_landing-audit-r6-body.md" "$s")" "$(count "$R/.claude/kb/checks-owed.md" "$s")"
}
row N01 '⑭'
row N02 '桶'
row N03 '混合大小写'
row N03 'summaryx'
row N04 '文件头第 36'
row N04 '另判形态'
row N05 '跨行'
row N06 'report-name-unchecked'
row N06 'no-cite'
row N07 'PIPESTATUS'
row N07 'set -e'
row N07 'bash -c'
row N08 '绝对路径核引文'
row N10 '验证段第 1 步'
row N11 'selftest-registry'
row N12 '内存上限'
row N13 'trust-dead-holder'
row N13 'memoryscan'
row N15 'CLAIM_SKIP'
row N17 '守卫判定之前'
row N18 'lib_writer_lock'
row N20 'ts3nxvwv'
row N21 '收拢表'
