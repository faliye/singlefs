# Verus 核接回 `crates/`：代码轮第二轮判决（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 S02 S03 S04 S05 S06 S07 S13 S14 K02 K19 K20 C540 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / diff | `research/prompts/_m3-verus-land-code-r2-body.md`、`_m3-verus-land-code-r2-background.md`、`_m3-verus-land-code-r2-diff.md` | — |
| 开工快照 | `research/prompts/m3-verus-land-code-r2-snapshot/`：`crates-sha256.txt`（215 份，冻结副本 `/tmp/claude-1000/m3-verus-land-code-r2/tree/`）、`kb-sha256.txt` | 冻结副本交齐之后 `sha256sum -c` 全 OK。`kb-sha256.txt` 对不上三份，都是别的会话改的：`.claude/kb/checks-owed.md` 与 `.claude/kb/milestone/03-third-txn.md` 是删除定案写回（改前改后清单 `/tmp/claude-1000/m3-writeback-c/report.md`「三份快照文件的改动清单」），两条腿一处都没引这两份；`.claude/kb/decisions/23-journal的角色与格式.md` 相对 HEAD 改在第 376、385、405、418、440 行，腿只引了第 355 行，在改动之前 |
| 云端正推（Sonnet） | `research/prompts/m3-verus-land-code-r2-sonnet-output.md`（161 行，sha256 37924c17…e404）；模型 `…-sonnet-model/` | 主 agent 在 `/tmp/claude-1000/m3-verus-land-code-r2-main-check/` 照 README 从冻结副本拷一份、`gen_old.sh` 生成改前四函数与 m1–m4、放 `zz_diff.rs`，经 `run.sh`（内存包装 8G、6 线程）跑：`random_family` 30 万个输入「Tally { total: 300000, diverged: 0, new_panics_only: 0, old_panics_only: 0, both_ok_differ: 0, panics_old: 0, panics_new: 0 …」；`positive_controls_are_red`「CONTROL[m1 去掉 take(limit)] diverged=1581 / 40000」「m2 … 3821」「m3 … 545」「m4 … 520」，四个数与报告逐个相同 |
| 云端攻方（Opus） | `research/prompts/m3-verus-land-code-r2-opus-output.md`（sha256 24a413ba…3834）；模型 `…-opus-model/` | Verus 变异与 H5-1「删掉之后仍 32 verified, 0 errors」没复跑：H5 那几处并进第 5 批实现员，落地时实现员与合入验证照样要跑 Verus |
| 引文 | 两份腿报告 | `python3 scripts/cite-check.py <两份报告> --root /tmp/claude-1000/m3-verus-land-code-r2/tree --root . --background research/prompts/_m3-verus-land-code-r2-background.md`：「✓ 核了 64 处引文，对上 64 处，没判 0 处」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| H1 改后核与 fcc4d340 等价 | 没打中：58 904 864 个产品输入零差异，改前四个变异体各抓到 1581、3821、545、520，对改后核 22 个单点改动全红；两条新断言在产品路径上触发不了（表按字段建键、计数器 6 字节）；越出产品输入时是契约收紧（键与字段不等、含 u64 最大值的表上改后 panic、改前返回，「两边都返回而结果不同」0），`# Panics` 已写明 | 没打中：8 处调用点建表都是键等于字段，计数器只有 48 位 | **没打中，两条腿同向，主 agent 复跑坐实**（第一节正推那一行） |
| H2 spec 补完之后证到了什么 | 不归 | 照字面判据没打中：自写 15 行只改可执行代码的变异，改结局的 6 行全红；照样验过的 8 行里 S02、S13、S14 等价，S03–S07 只改回调调几次、次序、对哪条调；`crates/mutations.tsv` 改核的 24 行跑 Verus 全红。口子：核头注释写「读盘次数与次序同原文」，spec 没写 | **没打中**。「读盘次数与次序」那一句：要么写进 spec（幽灵读盘日志），要么从核头注释删掉、改成由差分与 harness 用例担保——记账，并进第 5 批（零轮） |
| H3 报错路径与 G6 | 没打中：核只有两种失败，各映射到改前同名成员、内容整值相等；16 个胶水单点改动都红 | 没打中 | **没打中，两条腿同向** |
| H4 第 4 批新测试与变异 | K19 映射与两条断言有测试钉住 | 照字面判据没打中：新加 9 行与改写 2 行在副本 release 上 11 行全红、红在点名那一条；H4-c：S03、S05 在唯一一条数挂载读盘的用例上绿（S06 红在环读次数上）；H4-b：K20 那一行（第 1502 行）锚在 H5-1 的死子式上 | **没打中**。H4-c 记账（与 H2 那句同一件，补一条检查读盘次数的用例或 spec）；H4-b 随 H5-1 一起改锚点 |
| H5 编码纪律 | 不归 | 打中 8 处：中 2——`crates/singlefs-replay-prefix/src/lib.rs` 第 1070–1073、1090 行死分支加死子式（删后 Verus 仍 32 / 0，它量过）、核与 core 两套同名 newtype 与枚举只靠模块路径分开、胶水手抄映射；低 6——冗余判定、语义只写在注释、K19 用例注释与代码不符、K02 魔法数、手抄辅助函数、同一结局两个名字 | **打中**（引文 64 处都核对上）。并进第 5 批实现员：H5-1 删死子式连同 H4-b 改锚点、H5-3 两套同名类型改成说清角色的名字或一处定义（核不依赖 core，名字上分开）、低 6 处逐条改。**零轮** |
| H6 E165 `--pinned-cases` | 没打中：② ③ ④ 找状态、① 重算、与 `e165-pin-red-examples.py --replay` 的读法都对得上；168 行指纹对拍 167 行相同，不同的一行是装置不会输出的行首裸 `state=` | 没打中 | **没打中，两条腿同向** |

两条腿提的改法都只修错与补测试，没有一条是为性能改 core。

## 三、两条腿之外主 agent 记的

- 正推自报用 python3 `open('w')` 就地改过一次自己模型目录里的 `zz_diff.rs`，执行前拒绝的写法那道闸没拦：并进共用库盲区那一批（与第一轮攻方 `sed -i` 那一笔同族）。
- 正推读过之后 `scripts/e165-pin-red-examples.py` 被改过（3-12、3-13 在改它），它引的行号指入库拷贝 `model/e165-pin-red-examples.py.read`。

## 四、下一步

1. 第 5 批实现员（与 3-2 E165 重跑同一批之前）：H5 八处、H4-b 锚点、H2 / H4-c 那一句（spec 加读盘日志或补用例）。
2. 第 5 批合入之后开代码轮第三轮，只攻那一批的改法；第三轮之后不再开轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-01 不受影响：只被引作 spec 该写进去的口径 |
