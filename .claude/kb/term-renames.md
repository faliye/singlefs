# 术语改名对照表

全仓改过名的概念，旧名与新名的唯一权威对照。**别处一律用新名**——`research/scripts/sweep-term.py --check` 扫全仓，
除 `.claude/term-rename-exempt` 登记的几处之外出现旧名就判红。

这一份自己登记在那张豁免表里：写「X 改名成 Y」这句话必须写得出 X，而**旧名要有一处查得到，历史产物与旧提交里的字段才对得上**。
要查某个旧名今天叫什么，只看这一份；要查某次改名当天改了哪些文件、为什么改，看决策变更史当天那一条。

## 超级块 → 系统配置（2026-09-20 起，英文标识符与冻结目录 2026-09-21）

用户 2026-09-20 定案：盘上那块 481 字节的结构不叫「超级块」——本工程的索引树是普通的树，丢了能重建，没有「超级」的东西；
那块结构是**系统配置**。2026-09-21 定案：英文标识符、文件名与冻结目录一并改，「直到全工程都搜不出来」。

<!-- term-renames:table -->
一行一对，`research/scripts/sweep-term.py` 按这张表换、按这张表查；**加一行就自动受门禁管**。
「匹配」写 `整串` 或 `词边界`（后者用在缩写上，免得打中别的词里的那几个字母）。长的写在前面，免得短的先把长的吃掉一半。

| 旧 | 新 | 匹配 | 是什么 |
|---|---|---|---|
| 超级块 | 系统配置 | 整串 | 中文术语 |
| SUPER_BLOCK | SYSTEM_CONFIGURATION | 整串 | 常量名，带下划线 |
| SUPERBLOCK | SYSTEM_CONFIGURATION | 整串 | 常量名 |
| Super_Block | SystemConfiguration | 整串 | 大驼峰带下划线 |
| SuperBlock | SystemConfiguration | 整串 | 大驼峰，类型名 |
| Superblock | SystemConfiguration | 整串 | 大驼峰，b 小写的写法 |
| superBlock | systemConfiguration | 整串 | 小驼峰 |
| super_block | system_configuration | 整串 | 蛇形带下划线 |
| super-block | system-configuration | 整串 | 连字符形态（二进制名、产物文件名） |
| superblock | system_configuration | 整串 | 蛇形标识符、模块名、文件名 |
| sb_ | system_configuration_ | 词边界 | 缩写前缀（`sb_mac`） |
| _sb | _system_configuration | 词边界 | 缩写后缀（`tail_sb`） |
| sb | system_configuration | 词边界 | 裸缩写。`sb` 本是为 superblock 设的，概念没了缩写也去掉，同日从 SOP 的缩写表里删行 |

**跟着改的文件名**：`crates/singlefs-core/src/superblock.rs` → `system_configuration.rs`；五个实验装置、五张变异表、五个留存产物
（E100 / E115 / E124 / E126 / E147）。搬迁怎么做见 [.claude/rules/path-moves.md](../rules/path-moves.md)「怎么做」那一节。

**没改的**：`SUPERBLOCK_MAGIC` 的**值** `*b"SFSB"` 不动——那是写进盘上的魔数，改它等于改磁盘格式。常量名跟着改了。

**改名前的原件**：在 git 里。中文那一批的前一个状态见提交 `495bede` 之前；英文标识符与冻结目录那一批见 `4105e70` 之后的工作区改动。
留存产物与源码是同一次一起改的，所以 `replay.sh` 的逐字节比对改名前后都成立——这一条由改完的全量复跑证明，不是声称。

## 历史版本

### 2026-09-21

- 立本文件。此前新旧对照散在决策变更史 2026-09-20（其二十）那一条里，而那一条自己也在清扫范围内，改完就读不出旧名是什么。
