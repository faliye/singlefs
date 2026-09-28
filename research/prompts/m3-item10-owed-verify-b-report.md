# 欠账核实段乙：报告（2026-09-28）

逐行判定在同目录 `verdicts.md`（17 行，按派发次序）。初判员的 tsv / report 只当线索，每一格都重新现查过。

## 计数

命令：`awk -F' \\| ' '/^\| C/{print $2}' verdicts.md | sort | uniq -c`，原样输出：

```
     12 已还
      2 过期
      3 部分已还
```

未还 0，拿不准 0。

| 判定 | 编号 |
|---|---|
| 已还 | C363 C366 C378 C394 C404 C475 C481 C483 C484 C485 C490 C491 |
| 部分已还 | C478 C482 C487 |
| 过期 | C341 C377 |

## 部分已还那三行，新的「拦什么」开头

- C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间）：inode 树的根与内部节点（孩子是码 3 容器）头里的 key 区间，写路径（`crates/singlefs-core/src/transaction.rs` 的 `build_inode_tree_units`，第 5969 行注释「key 区间 = 第一条与最后一条**条目的 key**」）与 checker（`crates/singlefs-checker/src/walk.rs:1369`、`:1651` 仍自称「C478 还开着」）仍按首末两条分隔 key。多层码 2 树那一半已还。
- C482（只供测试的开关有三个走了哪一条看不出来）：① ② 在；③ 的「两臂分支名不许相同」那条 `assert_ne!` 在 `45d49aaf` 改写回退用例时被删掉，今天 `crates/mutations.tsv` 第 245 行多半不会红（推的，没跑）。
- C487（门禁 55 号拿活代码与不跟踪它的模型比）：E142 装置补上了 mkfs 清零（带断言与三条变异）；55 号段序列那一支没有判别力样本，且登记产物 mkfs 行（21 次、`[unit_write×4,zero_fill×8,barrier]`）与活代码（23 次、`[zero_fill×8,unit_write×4,barrier×2]`）不一致，55 号今天多半仍红（推的，没跑）。

## 拿不准

无。判得最不稳的两格写在这里：

- C366 判已还：行里「怎么拦」写的是「多次挂载的固定脚本里」，出处栏后来把会红的那一格改指「所选根自己那条记录两份都读不出」；那一格 C579 之后可写挂载被拒，会红的用例挪到 harness 的 `warm_up_journal_counter.rs:319`（B 一次两条记录、txg 与 jsn 分开）。主 agent 若认定必须落在层 0 固定脚本里，这一格改判部分已还。
- C363 判已还：会红的检查取的是 2026-09-17 改写后的形态（空发布实际写出的固定点槽数不许多于准入扣的 ckpt_cost，`admission_checkpoint_cost_per_device_paths.rs:690`），不是行里最早那句「把一棵记录树的高改高一层」。

## 顺带看到的（不在这件活的出口里，没改）

1. **门禁 52 号今天是红的**：只读跑 `nice -n 19 python3 research/scripts/check-segment-registry.py --root <仓根>` 退出码 1，报 2 处：mkfs 种根那一行的种类串（`layout/01-first-txn.md:400` 写 21 次、`barrier`，钉它的用例 `45d49aaf` 起是 23 次、`barrier×2`），与「第二条流」数组在 `crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs` 里找不到。原样输出在 `gate52-run.txt`。
2. 这批已还或过期的行，kb 与代码里仍有旧说法：
   - C341 相关：`05-快照-空间记账机制.md:133`「（已定项 2 的 K 代点删要求每行每发布重写）」；`23-journal的角色与格式.md:367` 末句「已定项 2 的点删随之改成「删掉 ≤ 当前代 − K 的全部代」」。
   - 欠栏仍列着的：`23-journal的角色与格式.md:464`（C378）、`19-块指针的结构与宽度预算.md:127`（C394）、`08-核心索引结构.md:85`（C490）、`23-journal的角色与格式.md:484`（C491）、`18-块里携带什么信息.md:61`（C478，这一条仍部分开着）。
   - C404 的依据句 `22-单元原子性怎么合成.md:183` 仍写「（推的，没量）」。
   - 代码注释：`crates/singlefs-core/src/allocator.rs:348`「（C475…那一半还开着）」；`crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs:12` 写「ckpt_cost 条款没给（C363…）」。
3. E142 装置源码（第十九次跑，`:7693` 写 23 次操作）已与 `research/scripts/replay.sh:171` 登记的第十八次产物对不上；登记是 `exact`，重跑多半会判不同（推的，没跑）。

## 什么现象会推翻这些判定

- 判已还的：把行里点名的那条变异打上去，点名的用例不红（今天没跑，全靠 `crates/mutations.tsv` 的登记与各自交回里的证红）；或点名的用例今天本身就红。
- C341 / C377 判过期：D5 已定项 2、D23 已定项 14 的回退形态再被改回「保留 K 代点删」「从 R_old 的账重载分配器」。
- C482 ③：第 245 行变异打上去之后 `without_the_shadow_ledger_…` 仍然红（说明别处还有断言盯着分支名）。

## 没做什么

- 没跑任何 cargo 编译或测试、变异、层 0、QEMU、herd7（派发提示禁跑；本机在跑全量）。凡说「会红 / 不会红 / 55 号仍红」的都是按代码读出来的，标了「推的，没跑」。
- 只跑了一次只读的 `check-segment-registry.py`（门禁 52 号的判定脚本，`admission: always`、不记指纹、不写仓）；没跑 `gate.sh`，也没跑别的门禁阶段。
- 崩溃验证（C378、C481、C491 涉及的 checker 档用例）没跑，也没核本机正在跑的那一趟的结果；按派发提示不计入判定。
- 没改仓里任何文件；上面「顺带看到的」各处旧说法没动。
- 草稿目录里除这两份外还有 `gate52-run.txt` 与三份 `verdicts-part*.draft`（拼 `verdicts.md` 用的中间件），没建编译目录、仓副本或工作树。
