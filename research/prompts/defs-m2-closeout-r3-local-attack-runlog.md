# 运行记录：defs-m2-closeout-r3-local-attack（2026-09-26）

提示文件：`research/prompts/defs-m2-closeout-r3-local-attack.md`（英文，K2 的 G7
攻击面：把十道阶段——27、34、40、69、75、84、85、86、88、99——各自「读不读实验页或
实验索引」「在 `.claude/gate.d/stage-owners.tsv` 里登记给不给执行员」拆成 T1–T10 逐格
核对：每项给一行来自该阶段脚本自己的原样引文（SRC-n）与一行来自 `stage-owners.tsv` 的
原样整行（REG-n），要模型按 SRC-CLASS / CLAIM-A / REG-FIELD-TWO / CLAIM-B 四字段逐格判，
外加 Q11–Q13 三道汇总与独立复核题，Q1–Q13 共 13 个标签）。
转述核对表：`research/prompts/defs-m2-closeout-r3-local-attack-translation-audit.md`。

调用方式：`nice -n 19 bash research/scripts/ask-local.sh <提示文件> > <前缀>-output-sN.md`，
每次前台起，未用 `setsid` / `&` / `disown`。跑之前 `ps -o pid,args -u "$(id -u)"`
查过 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`：零命中，
不是性能测量在跑。网关取 key 用的 `~/code/ai-center/.env.tenants` 存在；`curl` 探测
`:8200/v1/chat/completions`（GET，无请求体）返回 `http_code=405`，网关本身在监听。

每份判绿的样本另跑 `python3 research/scripts/oov-check.py <样本>`，并通读全文核有没有
缺词、断句一类损坏（列表里整个词没了、只剩孤零零标点这一类）。

## 逐次调用

1. 第一次调用（重定向到 `defs-m2-closeout-r3-local-attack-output-s1.md`）：`ask-local.sh`
   退出码 5。字词损坏闸判红：`corruption-check.py` 报「星号落单=1」（模型答复第 9 项的
   反驳句里原样引用了 `.claude/kb/**/*.md` 这个通配符，两个连续星号加后面的 `*.md`
   被闸认成未配对的星号标记）。脚本按规则把这一轮原样输出另存成
   `defs-m2-closeout-r3-local-attack-output-void1.md`（381 词，不计入干净样本，也不当
   「三方不一致」），`s1` 这次重定向建出的文件是空文件（0 字节）。判定：作废。
2. 第二次调用（沿用同一个号，重定向到同一个 `defs-m2-closeout-r3-local-attack-output-s1.md`，
   覆盖上一次的空文件）：`ask-local.sh` 退出码 0，386 词。
   `oov-check.py` 判绿：`生词=0 拼接=0`。通读全文：13 组答案齐全（T1–T10 逐项 + 汇总
   NONE + T1–T10 类别复述 + SRC-1–SRC-10 复核），每句完整、没有缺词，没有列表项只剩
   孤零零标点的情形，没有断句。判定：干净。
3. 第三次调用（重定向到新文件 `defs-m2-closeout-r3-local-attack-output-s2.md`）：
   `ask-local.sh` 退出码 0，359 词。`oov-check.py` 判绿：`生词=0 拼接=0`。通读全文：
   Q1–Q13 十三行齐全，每行四字段或对应格式完整，没有缺词、没有断句。判定：干净。

干净样本已达 2 份（`-s1.md`、`-s2.md`），按定义「攒到至少两份干净样本为止」在此停止，
未继续第四次调用。

## 汇总表

| 调用序号 | 目标文件 | 退出码 | 词数 | 生词清单（`oov-check.py`） | 判定 |
|---|---|---|---|---|---|
| 1 | `-output-void1.md`（`-output-s1.md` 这次是空文件） | 5 | 381（void 副本自身词数） | 未跑（闸在损坏阶段先判红退出，没跑到 oov-check 这一步；损坏类别是星号落单，不是生词） | 作废 |
| 2 | `-output-s1.md`（沿用同一个号重跑） | 0 | 386 | 空（`生词=0 拼接=0`） | 干净 |
| 3 | `-output-s2.md` | 0 | 359 | 空（`生词=0 拼接=0`） | 干净 |

## sha256sum

```
1c91104f70ac5f78f37019e2195a9ee342e6ace37647d3e584573933680c9664  research/prompts/defs-m2-closeout-r3-local-attack.md
da175b290088c863439a7cde9b024d20982f7d1cbbec6018169486ab84c0e792  research/prompts/defs-m2-closeout-r3-local-attack-output-s1.md
fb5fdbbecda6575b30ee043538c8d8c58e84dfea8a23e994c074fe0766519c5c  research/prompts/defs-m2-closeout-r3-local-attack-output-s2.md
12f9fb71551f2acf590875552b59e89f91910afd72ce8d9d30d69110d8b74178  research/prompts/defs-m2-closeout-r3-local-attack-output-void1.md
```

## 没做什么

- 没解读、没总结、没采纳样本 s1/s2 的具体答复内容；两份样本方向一不一致不在这份记录里判断。
- 没建 `research/prompts/defs-m2-closeout-r3-local-attack-model/` 探针目录：本地攻方定义
  「写范围」一节只列了提示文件、核对表、样本文件、运行记录、草稿目录，没有列 `-model/`
  探针目录这一项，不在这份定义的写范围内。
- 没跑第四、第五次调用：第二份干净样本在第三次调用就已拿到，按定义到此为止。
