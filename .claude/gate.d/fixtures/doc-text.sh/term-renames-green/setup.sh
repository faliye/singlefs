#!/usr/bin/env bash
# term-renames 那一格的绿样本现场（原是术语改名那一道的绿样本）：与红样本同一张改名登记表与排除表，rs 里只有新名。
# 旧名由几段拼进变量（登记表的旧名一列要写得出旧名），这份 setup.sh 与样本目录里都不出现旧名的字面。
set -e
old_chinese="超""级块"
old_constant="SUPER""BLOCK"
old_snake="super""block"
old_prefix="s""b_"
mkdir -p .claude/kb crates/singlefs-core/src
printf '%s\n' '# 判别力样本的豁免表' '.claude/kb/term-renames.md  # 表自己写着旧名' > .claude/term-rename-exempt
printf '%s\n' '# 判别力样本的改名登记表' '' '<!-- term-renames:table -->' '' \
  '| 旧 | 新 | 匹配 | 是什么 |' '|---|---|---|---|' \
  "| ${old_chinese} | 系统配置 | 整串 | 中文术语 |" \
  "| ${old_constant} | SYSTEM_CONFIGURATION | 整串 | 常量名 |" \
  "| ${old_snake} | system_configuration | 整串 | 蛇形标识符 |" \
  "| ${old_prefix} | system_configuration_ | 词边界 | 缩写前缀 |" \
  '' '## 历史版本' '' '### 2026-09-21' '' '- 首版。' > .claude/kb/term-renames.md
printf '%s\n' 'const SYSTEM_CONFIGURATION_BYTES: u64 = 481;' '// 系统配置槽宽 4096' 'let system_configuration_mac = 16;' > crates/singlefs-core/src/sample.rs
