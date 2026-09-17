# 变异分诊报告：e67_device_subset

- 输入：变异表 `research/mutations/e67_device_subset.tsv`；源文件 `research/e7-index-bench/src/bin/e67_device_subset.rs`；
  跑在仓副本 `/tmp/claude-1000/agents/trials/mutation-triage/repo/`（无 `.git`），未碰真仓
  `/home/fy5090/code/singlefs`。
- 改动前的三个数（主 agent 给）：抓到 3、无效 1、没红 0（2026-09-16 那次复跑）。

## 第 1 步：开跑前的 `ps` 检查

开跑前：

```
$ ps aux | grep -E 'cargo|qemu-system|gate\.sh' | grep -v grep
(无输出)
```

本机没有别的 `cargo` / `qemu-system` / `gate.sh` 在跑。紧接着 `nice -n 19 bash scripts/mutate.sh`
之前又确认了一次，同样零命中。派发提示给的例外（`CARGO_TARGET_DIR` 指到草稿目录、允许照常编译）
按 `nice -n 19` 用了。

## 第 2 步：逐条子串计数（第七类锚点腐化检查）

对表里 4 条的「原文」在源文件里做恰好一次的计数（`mutate.sh` 自己也这样判）：

| 变异名 | 命中次数 |
|---|---|
| M1_戊不固定每次重选 | 1 |
| M2_命中只看一块盘 | 1 |
| M3_窗口宽度写死 | 1 |
| M4_丢数据按w计 | 1 |

四条全部恰好命中一次，没有第七类（锚点腐化）问题，整张表可以跑。

## 第 3 步：跑 `mutate.sh`

### 输入本身有一处对不上：给的 bin 名与 `Cargo.toml` 声明不一致

按给定命令直接跑：

```
$ nice -n 19 bash scripts/mutate.sh e67_device_subset \
    e7-index-bench/src/bin/e67_device_subset.rs mutations/e67_device_subset.tsv
mutate: 源文件 e7-index-bench/src/bin/e67_device_subset.rs 在 Cargo.toml 里声明的二进制是 'e67-device-subset'，不是 'e67_device_subset'
        改的文件与跑的测试对不上，整份证明作废。用： mutate.sh e67-device-subset e7-index-bench/src/bin/e67_device_subset.rs mutations/e67_device_subset.tsv
EXIT_CODE=6
```

现查 `research/e7-index-bench/Cargo.toml` 第 144–146 行：

```
[[bin]]
name = "e67-device-subset"
path = "src/bin/e67_device_subset.rs"
```

`Cargo.toml` 里声明的二进制名是连字符 `e67-device-subset`，不是下划线 `e67_device_subset`——
派发提示给的「bin 名 `e67_device_subset`」与真实声明不一致。这不是我的判断，是 `mutate.sh`
自带的第 12–39 行那道自检（「先确认改的文件与跑的二进制对得上」）当场拦下并给出了唯一改法，
我按它给的命令改用 `e67-device-subset` 重跑（详见文末「试跑观察」）。

### 用 `e67-device-subset` 重跑，完整原样贴脚本汇总输出

```
基线：全绿
⏭  [M1_戊不固定每次重选] 变异导致编译失败，本条无效（不计入盲区，也不算命中）
✅ [M2_命中只看一块盘] 红：absolute_lost_fraction_is_three_over_28,absolute_object_pinned_intact_matches_pair_ratio,absolute_object_pinned_is_22_over_28,absolute_round_robin_intact_is_four_over_28,golden_pinned_by_emptiest
✅ [M3_窗口宽度写死] 红：skew_actually_skews,absolute_object_pinned_is_22_over_28,absolute_object_pinned_intact_matches_pair_ratio,absolute_round_robin_intact_is_four_over_28,absolute_lost_fraction_is_three_over_28,conservation_lost_fraction_equal_across_arms
✅ [M4_丢数据按w计] 红：absolute_lost_fraction_is_three_over_28
已还原，基线仍全绿
EXIT_CODE=0
```

## 第 4 步：整张表有没有跑完

收尾行是「已还原，基线仍全绿」，退出码 0——整张表 4 条全部跑完，不是中途退出，这一次的数算数。

## 第 5 步：三个数，与改动前比

| | 抓到 | 无效 | 没红 |
|---|---|---|---|
| 这次 | 3（M2、M3、M4） | 1（M1） | 0 |
| 改动前（2026-09-16） | 3 | 1 | 0 |

三个数与改动前逐项相同，「无效」没有变多，不需要单列。

## 第 6 步：没红 / 无效逐条分类

没红：0 条，无需分类。

无效：1 条，按 `.claude/rules/mutation-sampling.md` 分类如下。

### M1_戊不固定每次重选 —— 第八类（变异跑了但无效，替换编不过）

- **原文**：`Pick::PinnedByEmptiest => pinned.clone().unwrap_or_else(|| {`
- **替换文**：`Pick::PinnedByEmptiest => (|| {`
- **依据（现查编译输出，副本单独复现，命令与产物见「试跑观察」）**：

```
error[E0308]: `match` arms have incompatible types
   --> e7-index-bench/src/bin/e67_device_subset.rs:115:47
    |
106 |                   match pick {
    |                   ---------- `match` arms have incompatible types
107 |                       Pick::ObjectPinned => contiguous(obj % DEVS),
    |                                             ---------------------- this is found to be of type `Vec<usize>`
...
115 |                       Pick::PinnedByEmptiest => (|| {
    |  _______________________________________________^
116 | |                         let mut d: Vec<usize> = (0..DEVS).collect();
117 | |                         d.sort_by_key(|&i| (used[i], i));
118 | |                         d.truncate(width);
119 | |                         d
120 | |                     }),
    | |______________________^ expected `Vec<usize>`, found closure
```

  替换文去掉了 `.unwrap_or_else(...)` 这次调用，只留下 `(|| { ... })` 这个闭包字面量本身，
  没有被调用——`match` 的这一臂从产出 `Vec<usize>` 变成产出一个闭包，与其余分支类型不符，
  Rust 类型检查在编译期直接拒绝，测试进程根本没跑起来。

- **分类依据**：`.claude/rules/mutation-sampling.md`「第八类：变异跑了但无效——替换编不过，
  那条行为零覆盖」逐字：「一条无效变异等于它本要证明的那个行为今天零变异覆盖，要换一条真会改行为的
  补上」。M1 命名想测的行为是「戊（PinnedByEmptiest）如果不固定、每次都重选会怎样」——
  这条替换文没有实现这个行为变化，只是造出一个类型错误，所以**「戊不固定每次重选」这条行为
  今天零变异覆盖**，这是我的推论，交主 agent 核；这一步不属于我改表的范围。
- **这是推论，不是事实**：我没有替换出一条会真正改变行为的等价替代文本（那属于「补取样点」，
  按任务定义不归这次分诊改）；只是指出编译错误的确切原因和它对应到 mutation-sampling.md
  的哪一类。

## 没做什么

- 没有补断言、没有改变异表、没有改源码、没有改测试——按任务定义「写范围」只写了报告和草稿目录。
- M1 应该换一条什么样的替换文才能真正测到「戊不固定每次重选」这个行为，我没有去写；
  这是本轮之外的事，留给主 agent 判断是否另派任务。
- 「没红」这一类今天是 0 条，所以文中没有出现「取样点不敏感」或「等价变异」的判定——
  按任务要求，这两类判据（解跨整数边界的输入 / 写同值理由）本轮没有可判的对象。
- 分类结论（M1 归入第八类）是推论，依据是现查的编译器输出，交主 agent 核。

## 试跑观察

- **定义第 3 步给的命令行参数与仓里的事实不一致，而且不是我能提前从「输入」里看出来的**：
  派发提示写「bin 名 `e67_device_subset`」，但 `research/e7-index-bench/Cargo.toml:144-146`
  声明的是 `e67-device-subset`（连字符）。`mutate.sh` 自己的第 12–39 行有一道核对
  「改的文件与跑的二进制对不对得上」的自检，命中即退出码 6 并给出唯一改法——
  这次是它拦住并给出正确命令，我才按它的建议改用连字符重跑。
  ⇒ **定义没说清"bin 名"这一项该怎么核**：分诊脚本本身的「怎么做」一节写的是
  「跑 `nice -n 19 bash research/scripts/mutate.sh <bin 名> <源文件> <变异表>`」，
  照抄主 agent 给的名字直接跑会在第一步就被 `mutate.sh` 拦下（退出码 6，不是「基线就是红的」
  那种 exit 2，也不是变异没命中的 exit 3）——分诊定义没有写「bin 名对不上时怎么办」，
  只能靠脚本自己报的下一步命令去猜该不该照办。这次判断是：脚本给的是机械事实核对
  （Cargo.toml 里查得到），不是语义判断，所以照办；但如果以后遇到脚本给不出唯一改法的
  同类退出，分诊角色该拿它当「跑不动」直接报告，还是该按脚本提示自行纠正，定义没有说。
- **`mutate.sh` 的退出码有好几种，分诊定义的第 4 步只写了一种判据**：定义第 4 步说
  「收尾没有『已还原，基线仍全绿』就是中途退出」，这次退出码 6（bin 名对不上）发生在
  「基线：全绿」这一行都还没打印之前，比「中途退出」更早——脚本连基线都没跑。
  这次能按脚本给的改法继续跑完，但如果脚本这次没能给出唯一改法（比如两个候选二进制名都对不上），
  定义没写清楚这算不算「这一次的数不算」要如实报的那种情形，还是要归为「输入缺一样就不开工」
  （`.claude/agent-common.md` 第 10 行）——退出码 6 严格说不是「输入缺一样」，是「输入给错了」，
  两者在流程上是不是要区别对待，定义没有覆盖。
- **派发提示的例外条款只提到「允许照常编译」，没有提到 bin 名要不要按 Cargo.toml 校正**：
  例外原文是「这个 bin 小，允许你照常编译，命令加 nice -n 19；CARGO_TARGET_DIR 设到草稿目录里」，
  这条只解除了「开跑前先看 ps」（第 1 步）的限制，没有涉及第 3 步命令行参数本身；
  bin 名不对是与例外无关的独立问题，是照抄 `mutate.sh` 建议的命令解决的，不是靠这条例外解决的——
  这点在报告正文里区分清楚了，这里只是说明分类。
- **本次没有遇到的其它步骤都与定义相符**：`ps` 检查（第 1 步）、子串计数（第 2 步）、
  跑完整张表并确认收尾行（第 4 步）、三个数与改动前对比（第 5 步）都按定义原样做通，
  没有出现与定义对不上的地方。
