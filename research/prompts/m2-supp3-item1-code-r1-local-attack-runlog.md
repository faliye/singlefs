# 运行记录：m2-supp3-item1-code-r1 本地攻方

提示文件：`research/prompts/m2-supp3-item1-code-r1-local-attack.md`（前台跑，未用 setsid / `&` / disown）。
核对表：`research/prompts/m2-supp3-item1-code-r1-local-attack-translation-audit.md`。
模型：本机网关 `:8200`，模型 id `local`（`AI_CENTER_URL` 默认值，未覆盖）。

## 调用记录（按发生顺序，共 3 次调用，占 2 个编号）

| 序号 | 命令重定向到的文件 | 退出码 | 处置 |
|---|---|---|---|
| 1 | `m2-supp3-item1-code-r1-local-attack-output-s1.md` | 0 | 计入编号 s1 |
| 2 | `m2-supp3-item1-code-r1-local-attack-output-s2.md` | 5（判红作废） | 该次重定向是空文件（0 字节），沿用编号 s2 重跑；作废副本由脚本自动留成 `m2-supp3-item1-code-r1-local-attack-output-void1.md` |
| 3 | `m2-supp3-item1-code-r1-local-attack-output-s2.md`（同一编号重跑） | 0 | 计入编号 s2 |

调用 2 的判红原样输出（`corruption-check.py` 一行）：
`红 /tmp/tmp.q0JiFFvocG  cjk=0 words=2523 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=1 缩写自粘=0`，`实词自复读: precondition`（同一个 ≥5 字母的实词「precondition」连续出现两次）。

## 两个编号各自的详情

样本 s1（`m2-supp3-item1-code-r1-local-attack-output-s1.md`）：
- 退出码：0。
- 词数：`wc -w` 2008（`corruption-check.py` 自己数得 2002，与断词口径差异属该脚本正常范围）。
- `oov-check.py`（两个参数，样本 + 提示文件，与 `ask-local.sh` 内部调用同一种传法）：绿，生词 0，拼接 0。
- `oov-check.py`（只给样本一个参数，按派发提示字面的跑法）：红，生词 1（`TransactionOutput`），拼接 1（`TransactionOutput`=transaction+output）；核实：`TransactionOutput` 是提示原文里就有的 Rust 类型名（提示第 83、95 行原样出现四次），不是模型造的粘连词，两个参数版的判定与 `ask-local.sh` 内部判定一致（绿）；只给一个参数会把提示自带的驼峰式类型名当成拼接生词误判，这一点写清楚不当结论用。
- `corruption-check.py`：绿（汉字复读 0、英文复读 0、反引号/星号落单 0、粘连 0、实词自复读 0、缩写自粘 0）。
- 主 agent 通读一遍：没有缺词、断句、孤立标点这类损坏；四栏（Z5 的 a/b/c/d，Z1 的 1/2/3/4）每格都填了内容，没有空格或半句。
- 判定：干净。

样本 s2（`m2-supp3-item1-code-r1-local-attack-output-s2.md`，即调用 3 的产物）：
- 退出码：0（同编号调用 2 判红作废，不计入这份判定）。
- 词数：`wc -w` 1604（`corruption-check.py` 自己数得 1670）。
- `oov-check.py`（两个参数）：绿，生词 0，拼接 0。
- `oov-check.py`（一个参数）：红，生词 1、拼接 1，同样是 `TransactionOutput`（提示原文自带的类型名），核实为同一类误判、不计。
- `corruption-check.py`：绿。
- 主 agent 通读一遍：没有缺词、断句、孤立标点这类损坏；每格都填了内容。
- 判定：干净。

## 作废那一份的详情（不计入两份干净样本）

`m2-supp3-item1-code-r1-local-attack-output-void1.md`（脚本自动落盘，权限 600）：
- 词数：`wc -w` 2465。
- 触发退出码 5 的原因：`corruption-check.py` 判「实词自复读」——「precondition」连续出现两次。
- 未再对它跑 `oov-check.py` 或做逐句通读：这一份按规则整轮作废，不据它下任何结论。

## 结论

两份干净样本已够（s1、s2），停止调用；总调用次数 3（未到「连续五次调用拿不到两份干净的」停止线）。本报告不写两份样本答了什么、方向是否一致，按定义「产出」一节办。
