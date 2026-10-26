#!/usr/bin/env bash
# 红样本现场，文件全由这里现写：一份 md 五行，标签式「（会话 <号>，<日期>）」、当主语用的、上游三语仓的、
# 四位十六进制的、行尾的会话号各一行；一份 rs 里会话号带了放行标记却没写理由。每行都该点名。
# 这份源码本身不出现会话号：目录名与十六进制之间用 "" 断开——否则门禁扫样本现场时会先把这份 setup.sh 判红。
set -e
repo="single""fs"
mkdir -p records crates/demo/src
printf '%s\n' "### 第 4 批开工（会话 ${repo}-""21，2026-10-01）" "已发消息，${repo}-""e1 回话让它先恢复" \
  "SOP 会话 ${repo}-ai-sop-zh-""86 同步来副本" "会话 ${repo}-""e152 在跑全量" "改动归 ${repo}-""0c" > records/会话.md
printf '%s\n' "// 交给 ${repo}-""9e  // session-names:allow" "fn main() {}" > crates/demo/src/no_reason.rs
# 三方材料的会话号排除前缀只罩登记的那几轮：同族而没登记的轮（_m4-trunc-r3-）、轮名只是开头相同的（_m4-trunc-r10- 不归 _m4-trunc-r1-）、
# 登记了的轮的腿报告（不以 _ 起头，不是材料）各一份，每份一行，都该点名。
mkdir -p research/prompts
for material in _m4-trunc-r3-background _m4-trunc-r10-appendix m4-gc-r1-opus-output; do
  printf '%s\n' "| C1 | 立账（会话 ${repo}-""4f，2026-10-20） |" > "research/prompts/${material}.md"
done
