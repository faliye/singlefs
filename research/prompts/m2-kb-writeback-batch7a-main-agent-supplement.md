# kb 第七批前半：主 agent 对起草规格的补充三条（2026-09-27 JST 17:4x）

起草规格 `/tmp/claude-1000/kb-batch7a-drafter/spec.json`（sha256 03982423…）9 条照写；起草报告「要主 agent 判的点」三条主 agent 定如下，逐字旧串 / 新串，都在同一次写回里做。

## 补 1：I-7.13 行的状态列（在规格第 1 条写进之后再改）

- 文件：`.claude/kb/invariants.md`
- 旧串（规格第 1 条写进去的那一句，恰好一次）：`坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 里现在还没有改坏触发它的一份，`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 因此判红，补不补交主 agent 定）`
- 新串：`坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_invariant_and_every_other_is_not_applicable`：盘 1 槽 0 的加密类型改成 1、重算整槽校验和，只红 I-7.13、红在盘 1 偏移 0 那一槽，其余不变量全报不适用）`
- 依据：实现员报告 `research/prompts/m2-rev-i713-bad-image-implementer-report.md`（sha256 4106be3d…），补丁 2026-09-27 08:23 UTC 打进主工作区；用例现查在 `checker_known_bad_images.rs:2325`。

## 补 2：在用条数 79 → 80

- 文件：`.claude/kb/invariants.md`（第 17 行，`<!-- invariant-count -->` 之下）
- 旧串：`现共 79 条在用（编号至 I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉），另有六条退役，编号不回收）`
- 新串：`现共 80 条在用（编号至 I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉），另有六条退役，编号不回收）`
- 依据：I-7.13 立号（规格第 1 条）。

## 补 3：D22 第 336 行「A3-checker-2 在做」

- 文件：`.claude/kb/decisions/22-单元原子性怎么合成.md`
- 旧串：`越界即拒：checker 对越界槽报违例、实现整池拒挂载（用户 2026-09-27 JST 14:0x 定「整池拒」，A3-checker-2 在做）。`
- 新串：`越界即拒：checker 对越界槽报 I-7.13（系统配置池级字段在读者收的范围里）、该池其余不变量报不适用，实现整池拒挂载（用户 2026-09-27 JST 14:0x 定「整池拒」）。`
- 依据：同规格第 1 条；变更史那一条（规格第 8 条）的快查里补一句 D22 这一处。
