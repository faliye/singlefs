# m2-lines234-code-r1 本地攻方：运行记录

提示文件：`research/prompts/m2-lines234-code-r1-local-attack.md`
翻译核对表：`research/prompts/m2-lines234-code-r1-local-attack-translation-audit.md`
攻击面：22 个文件的测试覆盖那一维（分工见正文 `_m2-lines234-code-r1-body.md:63-71`）
全部调用前台跑，未用 `setsid` / `&` / `disown`。

## 调用记录（按发生顺序，退出码 0 的每次调用占一个号；判红作废的号不重复占用，沿用同一个号重跑）

### 第 1 次调用（判红，作废）

命令：`bash research/scripts/ask-local.sh research/prompts/m2-lines234-code-r1-local-attack.md > research/prompts/m2-lines234-code-r1-local-attack-output-s1.md`

退出码：5（`ask-local.sh` 内部字词损坏闸判红，按规则这一轮作废，脚本自动把
输出另存为 `research/prompts/m2-lines234-code-r1-local-attack-output-void1.md`，
原定向目标 `-output-s1.md` 是空文件，其后沿用同一个号 s1 重跑）。

判红原因（`corruption-check.py` 报告，命令与原样输出）：

    红 /tmp/tmp.JT3Au4lfsv  cjk=0 words=4444 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=20 实词自复读=0 缩写自粘=0
    红 research/prompts/m2-lines234-code-r1-local-attack.md  cjk=0 words=5573 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=20 实词自复读=0 缩写自粘=0

根因现查：`corruption-check.py` 的粘连（glue）判据
（`research/scripts/corruption-check.py:90`，正则
`(?<![A-Za-z0-9])[:;,]\w`）把 Rust 路径记号 `::` 的第二个冒号误判成「标点粘死
后词」——本地攻方当时的提示与模型跟着提示逐句复述出来的答复里各出现 19-20 处
`singlefs_core::模块名` / `singlefs_harness::模块名` 这类写法，两份各判红一次。
这不是模型输出真的字词损坏，是提示自己的写法触发了检测器的已知盲区（该检测器
本身没有对 `::` 记号单独开例外）。处置：用
`research/scripts/replace-batch.py` 对提示文件做 19 处定点替换（改写成
「imports items from the X module of the Y crate」这类不含 `::` 的说法），
`--dry-run` 核过 19 处命中都对之后落盘，回读一致；改完单独跑
`corruption-check.py` 核过提示本身判绿（`粘连=0`）之后才重新调用
`ask-local.sh`。改法记录：`research/prompts/m2-lines234-code-r1-local-attack-fix-colons.json`。

`oov-check.py` 对这份作废输出的判定（命令与原样输出）：

    绿 research/prompts/m2-lines234-code-r1-local-attack-output-void1.md  生词=30 拼接=0
         生词: mutations constant's checker's sandwiched undercounted disagreeing

拼接=0，`oov-check.py` 本身判绿；这份作废完全是 `corruption-check.py` 的粘连
判据触发的，不是 `oov-check.py` 那一类损坏。生词表以 Linux 内核词表为准，
`mutations` `constant's` `checker's` `sandwiched` `undercounted` `disagreeing`
都是真英文词或真词加所有格 / 分词后缀，不是缺头的粘连词，判绿成立。

作废，不计入观测，不计入两份干净样本的计数。

### 第 2 次调用（s1，干净）

命令：`bash research/scripts/ask-local.sh research/prompts/m2-lines234-code-r1-local-attack.md > research/prompts/m2-lines234-code-r1-local-attack-output-s1.md`

退出码：0
词数（`wc -w`）：4291（脚本自算 `words=4510`，两者算法不同——`wc -w`
按空白分词，`corruption-check.py` 按 `[A-Za-z][A-Za-z'-]*` 的正则算英文词，
两个数都如实抄，不取一个当唯一真值）

`corruption-check.py` 原样输出：

    绿 research/prompts/m2-lines234-code-r1-local-attack-output-s1.md  cjk=0 words=4510 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0

`oov-check.py` 原样输出（生词清单）：

    绿 research/prompts/m2-lines234-code-r1-local-attack-output-s1.md  生词=0 拼接=0

生词清单：空（零个）。

通读结果：22 行齐全，Row 1 到 Row 22 与末尾的 unknown 计数行都在，没有看到
缺词、断句、孤零标点这类肉眼可见的损坏。

判定：干净。

### 第 3 次调用（s2，干净）

命令：`bash research/scripts/ask-local.sh research/prompts/m2-lines234-code-r1-local-attack.md > research/prompts/m2-lines234-code-r1-local-attack-output-s2.md`

退出码：0
词数（`wc -w`）：4280（脚本自算 `words=4499`）

`corruption-check.py` 原样输出：

    绿 research/prompts/m2-lines234-code-r1-local-attack-output-s2.md  cjk=0 words=4499 fffd=0 汉字复读=0(0.00/千) 英文复读=0(0.00/千) 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0

`oov-check.py` 原样输出（生词清单）：

    绿 research/prompts/m2-lines234-code-r1-local-attack-output-s2.md  生词=0 拼接=0

生词清单：空（零个）。

通读结果：22 行齐全，Row 1 到 Row 22 与末尾的 unknown 计数行都在，没有看到
缺词、断句、孤零标点这类肉眼可见的损坏。

判定：干净。

## 小结

调用总数：3 次（1 次判红作废、2 次退出码 0）。
干净样本数：2（s1、s2），达到「至少两份干净样本」的停止条件，停止抽样。
带损坏样本数：0。
作废样本数：1（void1，退出码 5，见上）。

翻译核对表：`research/prompts/m2-lines234-code-r1-local-attack-translation-audit.md`

## 没做什么

- 不解读、不总结、不采纳两份样本各自答了什么，也不比较两份样本在列 d
  上是否方向一致——按定义与规则，本地攻方判它答得对不对、几份样本方向一不一致
  是主 agent 的事，不是本地攻方的事，这份运行记录只报退出码、词数、生词清单、
  干净 / 带损坏 / 作废。
- 没有验证模型答案里点的函数名、行标题是否真的存在于对应的源文件里——
  按定义本地攻方只负责忠实转达与收回，不核对模型答复的对错。
- `ask-local.sh` 内部对 `$TXT`（模型答复）与提示文件各自单独跑一次
  `corruption-check.py` / `oov-check.py`，第 1 次调用两个文件都判红；
  第 2、3 次调用只对模型答复单独复核过（如上），没有再对提示文件本身重复跑
  `corruption-check.py`——提示文件在两次复核之间没有再改动，第一次改完之后已
  确认判绿（见上「根因现查」一节），沿用同一份不再重复验证。
