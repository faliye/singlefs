# tooling-broad-mutation-prune-r2 辩方腿的模型与脚本（2026-10-01）

只读：`git show HEAD:<文件>`（HEAD 是 fcc4d340）、库目录 `~/.local/share/singlefs/broad-mutation/versions/19c45e02…/` 的 `list.json`、`results.tsv`、`baseline.tsv`、`logs/`。不起 cargo、不起驱动、不编译。
从本目录跑：`bash run_all.sh`，每个脚本的原样输出落在 `out/<脚本名>.txt`。跑完会多一个 `__pycache__`，可删。

| 脚本 | 做什么 |
|---|---|
| `lex2.py` | Rust 的词法切分与函数边界（token 级，不是 syn）；被别的脚本引 |
| `parse_list.py` | 清单 5091 条各落在哪个函数里；函数名对照变异名（0 处对不上）；产出 `rows.json` |
| `commits.py` | 最近 150 个非合并提交里动了清单文件的 15 个：被改文件名下条数、被改函数名下条数、函数外面的部分变没变（F2）；产出 `commits.json` |
| `res.py`、`t_inline.py` | 已判 302 条的结局、单价、T 的分布、T 与被改文件同文件的占比 |
| `constgraph.py`、`s2.py`、`s2b.py`、`constonly.py` | 常量与 const fn 的引用闭包（按项、按文件两种展开）、S2 的真 crates 射程、只在常量上下文被叫的 const fn |
| `fmtrefs.py`、`fmtitem.py`、`globals.sh`、`regimes.py` | format 的 lib.rs 被引与变更、全局项与读数据文件的变更、两种读法的逐提交重判条数 |
| `s4.py`、`derive.py` | 同名固有方法与 trait 方法、derive 定义文件的一跳范围 |
| `ratios2.py`、`grid.py`、`calm.py` | 变异那一趟里通过的测试用时与基线用时的比、按「倍数加下限」代进去的越限数 |
| `sim.py`、`v3.py` | V3 的随机抽函数与小时、编译次数的算术 |

限度：函数边界、常量与 derive 的引用都是名字匹配，不是类型解析；覆盖没做（本机没有覆盖工具、按定义不编译），所以关联文件的溢出只有上界。
