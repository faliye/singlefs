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
