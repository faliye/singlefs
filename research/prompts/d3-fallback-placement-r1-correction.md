# d3-fallback-placement-r1 判决的更正：第三、四节的老化盘读数作废（2026-11-10）

<!-- doc-lint:not-numbers L1 R -->

被更正的：`research/prompts/d3-fallback-placement-r1-main-verification.md` 第三节「老化盘读数」整张表、第四节「结论」与第五节「交用户」两行、第二节「只用用户操作的老化盘」一格。原件不改。

## 为什么作废

- 用户 2026-11-09 不接受第三节读数直接下结论，要求「重新量。确认是不是测量方法是不是有问题」。
- 方法复核（`~/.local/share/singlefs/work/d3-fallback-remeasure/result.md`，读码）判三处有问题：各臂各自挂载一次、起点差 2 槽（`aged_probe arm=today … free_slots=5121`，别的臂 `free_slots=5123`）；老化全程走今天的政策、不是臂自己老化的盘；老化全程不跑整理，被拒之后真实路径的出路（后台整理）模型里没有。被拒的格只能读作「不跑整理时写到底额的写数」。
- 重量探针（任务 11643、11648、11649、11670、11671、11672）跑前写死的判读线 L1 没过：212 格里 4 格池级 checker 报 `I-5.6`，其中有 `today` 臂，整批作废（`~/.local/share/singlefs/work/d3-fallback-remeasure-2/result.md`）。
- 诊断（任务 11789、11790）：未打补丁的树上同一格一样红（`checker_violated=1`，`end_digest=671366214263a8f2` 两棵树相同），违例出在整理因空间被拒暂停之后、前台写落进 R；与回落政策无关，是暂停撤 R 排他与 I-5.6 两条条款相撞（`~/.local/share/singlefs/work/d3-i56-diagnosis/result.md`）。用户 2026-11-10 定「整理暂停 删意图 下次重来」，实现在写。

## 现在的结论

- 第二节「优先重写集、不够退最低空槽，修不修得了撞 64」一格不受影响：锻造盘上落点逐个相同（`forged-sweep-today-vs-patched.diff` 0 字节；重量探针 `control_today` 两棵树摘要全等）。
- 先取重写集与各条退路在老化盘上谁好谁坏：没有读数。第四节「优先重写集不实现」与「能关上 64 的两条退路在老化盘上更差」两句撤回。
- D3（空间分配） 已定项 8 第 2 条不改，回落政策照今天。整理暂停删意图合入之后，照 `~/.local/share/singlefs/work/d3-fallback-remeasure/result.md` 的判读线重量一次，再交用户。
