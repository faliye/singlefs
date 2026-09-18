# 运行记录：m2-emptypool-nonempty-r1-local-attack

提示文件：`research/prompts/m2-emptypool-nonempty-r1-local-attack.md`
转述核对表：`research/prompts/m2-emptypool-nonempty-r1-local-attack-translation-audit.md`
前台调用命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-emptypool-nonempty-r1-local-attack.md`（未用 `setsid` / `&` / `disown`）

## 调用记录

| 调用序号 | 输出文件 | 退出码 | 词数（`wc -w`） | oov-check.py | corruption-check.py | 判定 |
|---|---|---|---|---|---|---|
| 1 | `m2-emptypool-nonempty-r1-local-attack-output-s1.md` | 0 | 1106 | 绿，生词=11（去重后：mutations, DiskSnapshot, unaccounted） | 绿，cjk=0 words=1123 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 干净 |
| 2 | `m2-emptypool-nonempty-r1-local-attack-output-s2.md` | 0 | 805 | 绿，生词=5（去重后：unmentioned, mutations, PoolReader） | 绿，cjk=0 words=831 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0 | 干净 |

两次调用都是退出码 0、两道字词损坏闸都绿、生词表里的生词逐一核过（`mutations`、`DiskSnapshot`、`unmentioned`、`PoolReader`、`unaccounted` 都是提示或答案里合法出现的领域词 / 派生词，不是粘连或断头），另外通读全文一遍未见缺词、断句或列表项缺失（提示要求的十条变异、九个测试、二十一格写覆盖判断、四条追问，逐条数过都齐）。两份都记为干净样本。无作废副本（本轮未触发退出码 5，`-output-void*.md` 不存在）。

连续调用两次即拿到两份干净样本，未用满五次上限。

## 备注

本轮未见「本地腿缺席」（网关可达，两次调用均取到正文）。
