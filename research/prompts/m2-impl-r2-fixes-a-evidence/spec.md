# 第二轮改法 A（checker 与 litmus 那一半，不撞在跑的实现员）

写于 2026-09-27（`TZ=Asia/Tokyo date` 现取）。来源与用户定案同 `/tmp/claude-1000/impl-r2-fixes/spec.md`（那份是整批；这一份只取不撞合入后验证一的文件：checker、坏镜像、一份新测试、litmus）。用户 2026-09-27 定「读字段，不等于常量就整池拒」「采纳，四个入口都核」、「开第三次代码三方的时候 解决 C577 加屏障的问题」（原话在 `records/2026-09-24-里程碑二收尾调度.md`）。代码三方第二轮判决 `research/prompts/m2-closeout-code-r2-main-verification.md`；攻方报告 `research/prompts/m2-closeout-code-r2-opus-output.md`（Y2-b 第 91–110 行、Y4 第 174–221 行、越格线索第 315–319 行）。

底座：主工作区现状（C577、A3b、善后一、I-9.16、两份坏镜像已合入；合入后验证一与第二轮改法 B 还没合入——它们改 core 的 `mount.rs`、`recovery.rs`、`transaction.rs`、`system_configuration.rs`，你不碰）。

## 做什么

| # | 关的是 | 改成什么 | 验收 |
|---|---|---|---|
| 1 | Y4-a：checker 的 I-7.13（系统配置池级字段在读者收的范围里） 只判格式版本、加密类型、槽距、根槽宽、环长；core 判 417 = 环长现算、起点在段边界；两边不是同一张表 | `crates/singlefs-checker/src/image.rs` 的 `judge_system_configuration_values_the_reader_accepts` 补三项：单元区起始槽号（偏移 417）= 环长现算的起点且在 64 槽段边界上；journal 环起点（偏移 325）= 第一版常量 1024；根环起点（偏移 371）= 第一版常量 64（常量从 `singlefs-format` 取，checker 那一份判定另写、不共用 core 的函数，门禁 94 号）；槽距上界改用同一槽自述的根环起点。core 那一半（读 325 / 371、不等整池拒）由第二轮改法 B 做，这一件不碰 core | 攻方 Y4 用例的 A、B、C、D、E 五格在 checker 上都报 I-7.13（`m2-closeout-code-r2-opus-model/opus_r2_y4_system_configuration_fields_core_and_checker_read_differently.rs`，只取它造镜像的写法，入库写成 `checker_known_bad_images.rs` 里的坏镜像：417 改 +64、环长改到段边界外、325 改 1023、371 改 65、371 改 0 各一份，各只红 I-7.13） |
| 2 | Y2-b 的越格线索：checker 读本盘设备号（`crates/singlefs-checker/src/lib.rs` 第 265 行附近），没有一条不变量拿它比盘的身份——盘体对调之后的池 checker 0 违例 | 池级 checker 新判一条：每块盘每个自证过的系统配置槽里的本盘设备号等于这块盘在池里的身份（D18（块里携带什么信息） 已定项 11 那张规格表的字段），不等报违例；编号暂立 **I-7.14**（照 I-7.13、I-9.16 的先例：`image.rs` 的 `IMPLEMENTED_INVARIANTS` 加一条，改号只改那一处），不变量原句写进报告交 kb 下一批 | `checker_known_bad_images.rs` 加一份「两块盘的本盘设备号互换、重算整槽校验和」的坏镜像只红 I-7.14；`the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` 绿 |
| 3 | C577 结清 ②：`litmus/` 的 P0 只写到根槽，不写系统配置槽与它之后的屏障（C577 报告「我不改、交主 agent 另派的」一节） | 补一条 litmus「轮换之后屏障」：P0 写根槽（FUA）→ 写系统配置槽 → 屏障 → 返回；观察者在返回之后读系统配置槽必须看到轮换后的值；配对照组（去掉屏障的 `-nofence` 版本，herd7 判 Sometimes / Never 要分得开），绑到 `transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`（锚点写法照 `litmus/` 已有六份与 `.claude/scripts/lkmm.sh` 的约定现查；`publish_order_matches_litmus.rs` 归第二轮改法 B 改）。herd7 是重型（57 号），你不跑；照已有 litmus 的形态写、静态核锚点 | 报告写明没跑 herd7、归提交时 57 号 |
| 4 | C577 结清 ③：屏障报错时零单元发布不冻结、整个挂载返回错误（D23（journal 的角色与格式） 已定项 14 射程今天已写成选择） | 一条用例钉住这一支今天的结局（零单元发布、轮换后屏障报错 → 挂载返回哪个错误成员、盘上哪几步已落、之后可写挂载怎么判），不改行为 | 用例绿；改坏（让它冻结）它就红，`prove-red.sh` 证一次 |

每处改法先红后改、一条「改回去它就红」的变异，`research/scripts/prove-red.sh` 证红；门禁 94 号跑到绿。

## 约束

- 要动的 crates 文件：`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker/src/lib.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`，新测试一份 `crates/singlefs-harness/tests/a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs`；`litmus/` 下新加两份（正例与 `-nofence` 对照）。份数不设上限。
- 同时在改 `crates/` 的，你都不碰：合入后验证一（core `mount.rs`、`recovery.rs`、`transaction.rs`，harness `model_comparison.rs`、`history.rs`、`bin/first_transaction_on_device.rs`、`bin/e158_root_choice_repair.rs`，几十份测试）；别的会话 singlefs-8b 在挪 11 份测试进 `crates/singlefs-checker/tests/`（层 0 文件与三份崩溃枚举用例）与两个包的 `Cargo.toml`。
- 交补丁：在副本里改，交 `patch/`（`crates.patch` 含 `litmus/`、`mutations-append.tsv`），交回前对主工作区 `git apply --check`。
- 重型测试：不跑。每条 `cargo test` 带 `--lib`、`--test <目标>` 或 `--bin` 之一。
- 线程上限：4。每条 cargo 经 `research/scripts/capped.sh 4`。
- 报告 `/tmp/claude-1000/impl-r2-fixes-a/report.md`；草稿目录 `/tmp/claude-1000/impl-r2-fixes-a/`；进度逐条追加进同目录的 `progress.md`。
