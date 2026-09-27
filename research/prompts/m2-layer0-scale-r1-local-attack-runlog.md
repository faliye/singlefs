# m2-layer0-scale-r1 本地攻方腿：运行记录

提示文件：`research/prompts/m2-layer0-scale-r1-local-attack.md`
核对表：`research/prompts/m2-layer0-scale-r1-local-attack-translation-audit.md`

## 调用记录

第 1 次调用（作废）：`bash research/scripts/ask-local.sh research/prompts/m2-layer0-scale-r1-local-attack.md > .../m2-layer0-scale-r1-local-attack-output-s1.md`，退出码 5。判红原因：`corruption-check.py` 与 `oov-check.py` 都是对 `$TXT`（模型答复）与 `$1`（提示文件本身）两份一起判的；这一次是提示文件自己被判红（提示里当时写的是 Rust 路径写法 `Script::ThirdVersion` 等三个标识符，`::` 被 `corruption-check.py` 的粘连正则 `(?<![A-Za-z0-9])[:;,]\w` 命中 4 次；同样这三个标识符被 `oov-check.py` 当成两个英文词粘死，`ThirdVersion` 报「拼接: ThirdVersion(=third+version)」），不是模型答复本身的损坏——模型那一份（$TXT）当次判绿（474 词，生词 5：`Falsified reproduces`，拼接 0）。作废副本落盘：`research/prompts/m2-layer0-scale-r1-local-attack-output-void1.md`（474 词，`oov-check.py` 判绿，生词 5：`Falsified reproduces`，拼接 0；这一份只是提示判红连带作废，其内容与之后 s1 的真实调用是两次独立观测，不通用）。改法：把提示里三处 `Script::X` 改写成不含双冒号的英文短语（`the third-version script` 等），改完再跑 `corruption-check.py`、`oov-check.py` 单独核过提示文件本身，两者都判绿后才重跑。

第 2 次调用：→ `m2-layer0-scale-r1-local-attack-output-s1.md`，退出码 0。474 词（`wc -w`；文件本身 2058 字节）。`oov-check.py` 判绿，生词 5（去重后 2 个：`Falsified`、`subtracts`），拼接 0。`corruption-check.py` 判绿（cjk=0 words=470 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0）。逐行通读：54 行表格逐行齐全，Q2/Q3/Q4 三段散文完整，每句都有主谓宾，无孤立标点、无缺词列表项。判定：干净。

第 3 次调用：→ `m2-layer0-scale-r1-local-attack-output-s2.md`，退出码 0。474 词。`oov-check.py` 判绿，生词 5（去重 2 个：`Falsified`、`segments'`），拼接 0。`corruption-check.py` 判绿（words=237 全项 0）。逐行通读发现 Q4 段第 2 句「For this to Table 4 Row 1's w_i values gives 5575802973, matching Table 4 Row 3.」读不通（"For X to Y gives Z" 不是合法英文句式，介词与动词之间明显掉了一个词，例如 applied/applying），两道自动检测器都没抓到（既非粘连也非登记过的生词形态）。判定：带损坏（人工通读发现的缺词/断句，两项自动闸都判绿）。

第 4 次调用：→ `m2-layer0-scale-r1-local-attack-output-s3.md`，退出码 0。476 词。`oov-check.py` 判绿，生词 6（去重 3 个：`Falsified`、`baseline's`、`reproduces`），拼接 0。`corruption-check.py` 判绿（words=251 全项 0）。逐行通读：Q4 段有一句「Applying This reproduces the value 5575802973 given in Table 4 Row 3.」，"This" 句中大写是本地量化模型常见的大小写小毛病，但整句主谓宾齐全（Applying this [作动名词主语] reproduces [谓语] the value...[宾语]），不属于「整词消失、只剩孤零零标点」那一类，其余 53 行表格与 Q1–Q3 全部齐整。判定：干净。

第 5 次调用：→ `m2-layer0-scale-r1-local-attack-output-s4.md`，退出码 0。450 词。`oov-check.py` 判绿，生词 5（去重 2 个：`Falsified`、`subtracts`），拼接 0。`corruption-check.py` 判绿（words=220 全项 0）。逐行通读：54 行表格齐整，Q2/Q4 散文完整、无缺词断句；Q3 给出的总数与另外三份不同（这属于答案内容层面的差异，不是词形损坏，按「不解读、不判它答得对不对」不在这里评判）。判定：干净。

## 汇总

- 干净：s1、s3、s4（3 份，达到「至少两份干净样本」的停机条件）。
- 带损坏：s2（1 份，人工通读发现的缺词/断句，两道自动闸门都判绿）。
- 作废：第 1 次调用（提示文件自身触发粘连闸，见上；已在第 2 次调用前改掉提示、两道闸门对提示文件单独核过绿再重跑）。
- 核对表：`research/prompts/m2-layer0-scale-r1-local-attack-translation-audit.md`。
