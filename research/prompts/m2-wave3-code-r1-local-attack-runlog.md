# m2-wave3-code-r1 本地攻方：运行记录

提示文件：`research/prompts/m2-wave3-code-r1-local-attack.md`
转述核对表：`research/prompts/m2-wave3-code-r1-local-attack-translation-audit.md`

## 调用记录（按发生顺序，共 3 次调用；判红作废的那次也计入次数）

1. 第 1 次调用。命令：`nice -n 19 bash research/scripts/ask-local.sh research/prompts/m2-wave3-code-r1-local-attack.md > research/prompts/m2-wave3-code-r1-local-attack-output-s1.md`。退出码 5（判红作废）。判红的不是模型答复本身：`corruption-check.py` 对模型答复（768 词）判绿，但对**提示文件自身**判红（`粘连=8`）——8 处都是 Rust 的 `::` 路径分隔符被「标点紧贴下一个词」这条启发式规则误判成粘连（例如 `RecoveryOutcome::FileRead`）。按脚本约定，重定向建出的 `m2-wave3-code-r1-local-attack-output-s1.md` 是空文件（0 字节），模型这次的原样答复被脚本另存为 `m2-wave3-code-r1-local-attack-output-void1.md`（4539 字节，这一份不算样本）。随后把提示文件里全部 8 处 `::` 改写成 `..`（并在提示的 Ground rules 里加一条说明这个记号），修好之后 `corruption-check.py` 对提示文件本身判绿（`粘连=0`）。这一次调用不算样本，也不算「本地腿缺席」——网关本身工作正常，只是这一轮的原始提示触发了判红闸，按规则作废重跑。

2. 第 2 次调用（沿用 s1 这个号，因为上一次判红时 s1 是空文件）。命令同上，提示文件已修好。退出码 0。输出：`research/prompts/m2-wave3-code-r1-local-attack-output-s1.md`。
   - 词数：`wc -w` 977；`corruption-check.py` 自己数得 1071（分词方式不同）。
   - `oov-check.py`（用提示文件做已知词表）：生词=2（`inequality`、`BaseImageTier's`），拼接=0。
   - `corruption-check.py`：cjk=0 words=1071 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0。
   - 通读一遍：14 道题全部作答，每道题格式完整（编号、列标签、落回条件句都在），没有看到缺词或断句（没有列表里整词消失、只剩孤零零标点的情况）。
   - 判定：干净。

3. 第 3 次调用（新开 s2）。命令：同上，输出改为 `research/prompts/m2-wave3-code-r1-local-attack-output-s2.md`。退出码 0。
   - 词数：`wc -w` 496；`corruption-check.py` 自己数得 517。
   - `oov-check.py`：生词=0，拼接=0。
   - `corruption-check.py`：cjk=0 words=517 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0。
   - 通读一遍：14 道题全部作答；后半段（问题 6 起）部分格的落回条件句被省略或答得更短，但没有出现缺词、粘连词或断句性的孤立标点，是答得简略，不是字词损坏。
   - 判定：干净。

## 小结

3 次调用，1 次判红作废（不算样本），2 次退出码 0 且两道字词损坏闸都判绿、通读也没发现带损坏的迹象 —— 达到「至少两份干净样本」，停止取样。
