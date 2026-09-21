# 样本记账表（红）

判别力样本，不是真的记账表。位号的唯一登记位是 D15（格式冻结政策） 已定项 4 的登记表，这份样本故意与它对不上。

<!-- feature-bits:table -->

| 位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话 |
|---|---|---|---|---|---|---|
| 0 | incompat | INCOMPAT_SAMPLE_LINE_BIT | 格式版本 1 | 0000000 | 在用 | 样本布局线 |
| 2 | incompat | INCOMPAT_SAMPLE_SECOND_BIT | 格式版本 1 | 0000001 | 在用 | 样本第二位 |
| 2 | incompat | INCOMPAT_SAMPLE_RECYCLED_BIT | 格式版本 2 | 0000002 | 在用 | 样本第二位回收之后改写的新语义 |

## 历史版本

### 2026-09-21

- 建档。
