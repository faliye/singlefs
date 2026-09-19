# 书记员报告：m2-closeout-kb-scribe

## 规格文件

- `research/prompts/m2-closeout-kb-scribe/spec.md`
- `research/prompts/m2-closeout-kb-scribe/owed-spec.json`

写范围闸对 `.claude/kb/checks-owed.md` 放行（`write-guard.sh` 退出码 0）。

## 开工时 sha256sum

```
03d34533f08eb3dcb135816f70cb67d514b59edbdd4e49e4fbc208f7a6d7bee9  .claude/kb/checks-owed.md
a40360a68380c0a41641c4a29763b67467dc0e4e13f9b4b2265011bbf2549dcb  .claude/kb/decisions.md
5392e631ff67fde7d54fac1d9e8b3987396397d935585c3723a6f5a5d1f3c0a1  .claude/kb/decisions-history.md
b0791d8ff554baa1811475fc74a3197759f35c5bfcf316762928ad40a1e3e471  .claude/kb/decisions-history/2026-09.md
```

这一轮不记决策变更史、不翻分项状态，`decisions.md` / `decisions-history.md` / `decisions-history/2026-09.md` 全程没写，收尾哈希不变，不再重复贴。

## 取号

写批处理规格之前现查欠账表最大编号：`grep -no 'C[0-9]\+' .claude/kb/checks-owed.md | sed 's/.*C//' | sort -n | uniq | tail` 到 392；`grep -rn 'C393\|C394\|C395' .claude/kb .claude/kb 与 research/` 零命中（唯一提到的是 spec.md 自己那句取号说明）。spec 给的占位号 C385/C386/C387 已被 2026-09-19（其四）（其五）两笔占用（分别是「首次写要不要读回」「节点大小结论欠一次复跑」「常量模块手写」），顺延取 **C393、C394、C395**：

| 占位号 | 简称 | 实取号 | 接在哪一行之后 |
|---|---|---|---|
| C385 | 被抛弃根的账读不出时修复没有条款 | **C393** | 开着那张表最后一行 C392（双轨翻转入口的穷举表没推）之后 |
| C386 | 释放判定不核映射条目位置项里的单元校验和 | **C394** | 紧接 C393 之后 |
| C387 | 代字段只有验收断言盯着 | **C395** | 紧接 C394 之后 |

C22、C369 两行还清：从开着的表删掉整行，插进「已还清」表 C382（研究脚本与 hook 的拒绝不在门禁自检的射程里）那一整行之后，顺序 C22 在前、C369 在后（照规格给的顺序）。

## replace-batch.py 原样输出

规格文件：`/tmp/claude-1000/m2-closeout-kb-scribe/replace-spec.json`（9 处编辑：5 处原地改写 + 2 处删行 + 1 处插入已还清两行 + 1 处插入开着表三行）。

```
$ nice -n 19 python3 research/scripts/replace-batch.py --dry-run /tmp/claude-1000/m2-closeout-kb-scribe/replace-spec.json
✓ 9 处都命中得对（1 个文件），--dry-run 没写

$ nice -n 19 python3 research/scripts/replace-batch.py /tmp/claude-1000/m2-closeout-kb-scribe/replace-spec.json
✓ 9 处替换，写了 1 个文件，回读一致
```

9 处逐条：

1. C77 怎么拦那一格补跨实例形态已有、还欠两样（原地改写，命中 1 次）
2. C42 前置：待代码三方已过（原地改写，命中 1 次）
3. C143 前置：另一半已答（原地改写，命中 1 次）
4. C378 改写成剩下的题面（原地改写，命中 1 次）
5. C381 前置：第一轮已判（原地改写，命中 1 次，规格第四节）
6. 删 C22 整行（开着的表）
7. 删 C369 整行（开着的表）
8. C382 那一整行之后插入 C22、C369 两行的还清说明（已还清表）
9. C392 那一整行之后插入 C393、C394、C395 三行（开着的表）

写完逐条回读核过（`grep -n '^| C22 \|^| C369 \|^| C393 \|^| C394 \|^| C395 \|^| C382 \|^| C392 '`）：C22、C369 只出现在已还清表（377 行之后），开着的表（359 行之前）里两个编号都搜不到；C393/C394/C395 紧跟在 C392 之后（356–359 行）；C77/C42/C143/C378/C381 五行都是新串。

## 门禁

### 43 号（欠账表两张登记表的行形状）

```
  ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 378 行，2 张表）
```
退出码 0。

### doc-lint.sh

```
  ✗ 文档铁律检查失败：1 个文件违规、0 处编号引用无定义、5 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 442，跳过 0）
```
退出码 1，红。逐条核过归属：

| 违规 | 位置 | 是不是这一轮写的 |
|---|---|---|
| kb 正文用上下文指代 | `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md:50` | 不是——该文件是别的会话未提交的改动（`git diff --stat` 显示改了 46 行，我没碰过这个文件） |
| `E155` 裸引用 | `.claude/kb/experiments-history.md:46` | 不是——同一个别的会话的未提交改动 |
| `E155`/`D25`/`A2` 等 6 处裸引用 | `.claude/kb/experiments/155-…md` | 不是——同上 |
| **`I-3.9` 裸引用** | `.claude/kb/checks-owed.md:359`（C395 行，「I-3.9 那一处补一份组合坏镜像」） | **是**——原文逐字来自 `spec.md` 第 30 行（C387 占位那一行），我按规格逐字誊写，没有加字 |
| **`I-4.8` 裸引用** | `.claude/kb/checks-owed.md:379`（C22 还清行，「`crates/mutations.tsv` 第 110 行（I-4.8 判定恒真）」） | **是**——原文逐字来自 `spec.md` 第 19 行（C22 那一行），同样逐字誊写 |
| 编号 `Y1` 出现 3 次却无登记位 | 3 处里 1 处在 `.claude/kb/checks-owed.md:358`（C394 行，「Y1 的射程」），另 2 处在 `.claude/kb/milestone/02-second-txn.md`（162、322 行，不是我写的） | 我这一轮补的那 1 处**是**逐字来自 `spec.md` 第 29 行（C386 占位），但触发「≥3 次」这条判据要另外两处配合——那两处不在我的写范围、我没碰 |
| 编号 `Y5` 出现 3 次却无登记位 | 3 处里 1 处在 `.claude/kb/checks-owed.md:51`（C42 行，「判决第 14 行 Y5 没算打中」），另 2 处在 `.claude/kb/milestone/02-second-txn.md`（328、333 行，不是我写的） | 我这一轮补的那 1 处**是**逐字来自 `owed-spec.json` 第 2 组新串，同样触发点在别处 |

`checks-owed.md` 里这 4 处红全部是规格给的原文逐字誊写，我没有替换或改写，也不许自己加简称去圆过去（规格里没写的句子一个字不加）。按定义第 4 步的判据，停在这里交主 agent 改规格：`I-3.9`、`I-4.8` 两处要各自补成「I-3.9（简称）」「I-4.8（简称）」的形态，`Y1`、`Y5` 要么在某处补登记位、要么在 `.claude/kb/milestone/02-second-txn.md`（不在我的写范围）里也补上简称——这两件事规格没交代怎么办，我不能替规格做主。

### 10 号（kb 腐化）

```
  ✓ 欠检查 343 条、已还清 35 条（checks-owed.md）
  ✓ kb 腐化审计通过
```
退出码 0。

## 改过的文件与前后 sha256

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/checks-owed.md` | `03d34533f08eb3dcb135816f70cb67d514b59edbdd4e49e4fbc208f7a6d7bee9` | `0a8b9d8c12b1146c541c6da11d8f9452ae1965093e61a043dabbf06c60bacdb7` |

只改了这一个文件。

## 没做什么

- 没记决策变更史（规格明写不记）。
- 没翻分项状态（规格明写不翻）。
- 没判 doc-lint 红的那 4 处（`checks-owed.md` 里的 `I-3.9`/`I-4.8`/`Y1`/`Y5`）该怎么补简称——那是规格给的原文，我没有自己改句子，停下交回。
- 没修 `.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md` 与 `.claude/kb/experiments-history.md` 里的红——不是这一轮的改动，也不在我的写范围里那两处对应的责任段。
- 没跑除 43、doc-lint、10 号之外的门禁阶段（这次任务只点名这三个）。
- 没提交。
