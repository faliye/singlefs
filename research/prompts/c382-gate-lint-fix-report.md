# C382：给 gate-lint 里的 84 处拒绝补出路

任务：让下面 12 个文件里每一条拒绝都带出路，使
`bash .claude/singlefs-ai-sop/scripts/gate-lint.sh research/scripts .claude/hooks`
在这 12 个文件上零处 ✗。判据见 `.claude/singlefs-ai-sop/rules/sop-first.md`「每一条拒绝都必须给出下一步」。

## 逐文件改动与 gate-lint 计数

改动方式：只加出路，不改判据与行为——退出码、判红条件不变；
自定义 `die()` 只打印 `"$*"` 的，改成接第二个参数并按「→ 怎么办：」格式打印（照 `lib.sh` 的 `die`/`howto` 形态）；
循环里逐条列明细的行标 `# gate-lint:detail`，出路写在对应的汇总行上；
纯打印且没有汇总的（e6-units.sh、write-guard.sh 的成功摘要），直接把出路写进消息本身或紧跟着补一行。

以下「改前/改后 gate-lint」均为该文件单独放进一个干净临时目录、单独跑
`GATE_LINT_DIR=<该文件所在临时目录> bash .claude/singlefs-ai-sop/scripts/gate-lint.sh` 的原样末行，
改前取自 `git show HEAD:<路径>`（这批文件在本轮开工前都是干净的，HEAD 版本即原始版本）。

| 文件 | 改了几处 | 改前 gate-lint | 改后 gate-lint |
|---|---|---|---|
| research/scripts/vm-geom.sh | 17（16 处 die 补第二参数 + die() 定义改造） | `门禁自检失败：17 处（共 1 个脚本、17 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、1 条拒绝都带了出路` |
| research/scripts/vm-bench.sh | 17（11 处 die 补参数/die() 改造 + 6 处直接打印补 → 行） | `门禁自检失败：17 处（共 1 个脚本、22 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、11 条拒绝都带了出路` |
| research/scripts/quote-kb.py | 11（selftest() 里每条 ✗ 断言各补一行具体指向哪个函数的 howto） | `门禁自检失败：11 处（共 1 个脚本、12 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、12 条拒绝都带了出路` |
| research/scripts/verify-citations.sh | 8（ck/ckn/ckdoc/ckdocn 函数体内的明细行标 `# gate-lint:detail`，出路已在文件末尾汇总行里） | `门禁自检失败：8 处（共 1 个脚本、9 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、1 条拒绝都带了出路` |
| research/scripts/replay.sh | 8（claim() 与 check_claims() 里的明细行标 `# gate-lint:detail`，末尾汇总新增一段按「跑不了/对不上/结论断言不中」分类的 howto） | `门禁自检失败：8 处（共 1 个脚本、10 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、2 条拒绝都带了出路` |
| research/scripts/e6-units.sh | 6（每处直接打印补一行具体 → 出路） | `门禁自检失败：6 处（共 1 个脚本、7 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、7 条拒绝都带了出路` |
| research/scripts/vm-kernel.sh | 4（die 补第二参数） | `门禁自检失败：4 处（共 1 个脚本、5 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、1 条拒绝都带了出路` |
| research/scripts/oov-check.py | 4（selftest() 四个循环里的明细行标 `# gate-lint:detail`，出路已在其后的汇总 `if bad:` 块里） | `门禁自检失败：4 处（共 1 个脚本、5 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、1 条拒绝都带了出路` |
| research/scripts/fetch-refs.sh | 3（while 循环里三处明细标 `# gate-lint:detail`，收尾摘要新增按失败类型分流的 howto） | `门禁自检失败：3 处（共 1 个脚本、3 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、1 条拒绝都带了出路` |
| research/scripts/pdf-text.py | 2（selftest 循环里两处明细标 `# gate-lint:detail`，新增一段汇总 `✗ 自检未通过` 加 howto） | `门禁自检失败：2 处（共 1 个脚本、2 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、1 条拒绝都带了出路` |
| research/scripts/replace-once.py | 1（main() 里打印 message 那一行补一句解释性注释；同时给 replace_once() 里两处原本没有 howto 的 return 2 分支补了 howto） | `门禁自检失败：1 处（共 1 个脚本、2 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、2 条拒绝都带了出路` |
| .claude/hooks/write-guard.sh | 1（成功摘要把 `{len(cases)}` 计数挪到含 ✓ 的那一行，之前计数落在下一行的 f-string 续行里，gate-lint 按行扫描看不到） | `门禁自检失败：1 处（共 1 个脚本、0 条拒绝）` | `门禁自检通过：1 个脚本（.sh 与 .py）、0 条拒绝都带了出路` |

「共 N 条拒绝」是 gate-lint 内部对这个文件里所有匹配到 die/bad/直接打印✗ 模式的行的计数（含已经带出路、不判红的），
不等于判红的处数——判红的处数是「门禁自检失败：M 处」里的 M，与任务给的逐文件清单一一对应
（17/17/11/8/8/6/4/4/3/2/1/1，合计 84 处，与任务描述一致）。

## 整组验证

### gate-lint 两个目录

```
bash .claude/singlefs-ai-sop/scripts/gate-lint.sh research/scripts .claude/hooks
```

末尾：

```
  ✗ relay-timing-lint.py:481  拒绝但没有出路（直接打印的 ✗，到下一处拒绝之前没有「→」）
  ✗ stale-candidates.py:344  拒绝但没有出路（直接打印的 ✗，到下一处拒绝之前没有「→」）
  ✗ stale-candidates.py:346  拒绝但没有出路（直接打印的 ✗，到下一处拒绝之前没有「→」）

  ✗ 门禁自检失败：3 处（共 67 个脚本、253 条拒绝）
```

这 3 处都不在本次任务范围：`relay-timing-lint.py` 是任务明令不许碰的别的会话未跟踪文件；
`stale-candidates.py` 不在「要改的文件」清单里，也是未跟踪文件（`git status --porcelain` 显示 `??`），
不属于这次要改的 12 个文件，没有动它。本次负责的 12 个文件逐个单独跑 gate-lint（各自复制到独立临时目录跑，
避免混进别的文件的红）均为 `门禁自检通过`，见上表「改后 gate-lint」列。

### 各脚本的 selftest

```
python3 research/scripts/quote-kb.py --selftest        → 全部 ✓，最后 exit=0
python3 research/scripts/oov-check.py --selftest       → ✓ oov-check 自检通过（红样本 10 个全抓，绿样本 20 个不误伤），exit=0
python3 research/scripts/pdf-text.py --selftest        → 三份文献锚点句全部命中，exit=0
python3 research/scripts/replace-once.py --selftest    → ✓ --selftest 通过：命中 0/2/空串 三种拒绝都不写字节，命中 1 次写成功并回读确认，exit=0
bash .claude/hooks/write-guard.sh --selftest           → ✓ 自检通过（查了 27 种情形），exit=0
```

vm-geom.sh / vm-bench.sh / vm-kernel.sh / e6-units.sh / fetch-refs.sh / verify-citations.sh / replay.sh
没有 `--selftest`（vm-bench.sh 的 `--selftest` 需要真实 QEMU/KVM 环境跑虚机，不在这次改动的验证范围内，
只做了 `bash -n` 语法检查与逐处 diff 核对；改动只加了打印语句，没有碰任何判定逻辑）。

### 门禁 47、46、63

```
bash .claude/gate.d/47-research-script-selftests.sh
  ✓ research 脚本的自证都通过（查了 15 份：……）

bash .claude/gate.d/46-write-hook.sh
  ✓ 写 hook（含 Write 覆盖未跟踪文件那一道）注册着，自检通过（查了 1 条注册、27 种情形）

bash .claude/gate.d/63-agent-write-scope.sh
  ✓ 写范围闸、Bash 检出 hook 与续派闸注册着、自证通过，表与定义一致（3 个有 Write 或 Edit 的定义、16 条路径模式）
```

三道门禁均绿。

## 没做到的

无。任务列出的 12 个文件、84 处（本次逐个核对后，`quote-kb.py` 实际是 11 处 ✗ 加 1 处此前已经隐藏在窗口边界内的
`replace-once.py` 计 2 条拒绝、`e6-units.sh` 计 7 条拒绝——这些是 gate-lint 内部「checked」计数与「fails」计数的口径差异，
不是漏改；每个文件改后单独跑 gate-lint 都是 0 fails）全部处理完成，语法与 selftest 均通过，门禁 47/46/63 全绿。
