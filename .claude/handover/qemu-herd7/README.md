# 从 singlefs-ai-sop 移交过来的 QEMU 与 herd7

从 2026-09-16 起，singlefs-ai-sop 不再管 QEMU 虚机与 herd7 / LKMM。这两样只有 singlefs 在用，
怎么测、怎么验、接不接进门禁，都由 singlefs 自己定。SOP 仓里相关的脚本、样本、模板与规则段落一并删掉，
删之前的原样放在 `sop-0.0.50-snapshot/`，取自 singlefs-ai-sop-zh 的提交 f660a91。

这个目录本身不接进门禁：`.claude/doc-lint-exclude` 里加了一行绕开它。规则是 SOP 的原文，
样本里有故意写坏的 litmus，接之前扫它只会报出不属于本项目的判定。

## 这批文件是什么

| 路径（相对 `sop-0.0.50-snapshot/`） | 原来在 SOP 里做什么 |
|---|---|
| `scripts/lkmm.sh` | 共享门禁的「LKMM」阶段：静态检查（rN 声明、atomic_t 初值、每条 Never 配内容对得上的对照组、`singlefs-models` 绑代码）加 herd7 判定 |
| `scripts/fetch-deps.sh` | 给 herd7 准备内核树里的 `tools/memory-model`，缺了就联网克隆 |
| `scripts/env.sh` | 环境自检。其中 `qemu-system-x86_64`、`dmsetup`、`fio`、`/dev/kvm`、`python3` 那几项是为 QEMU 与 lkmm 查的 |
| `scripts/fixtures/lkmm/` | `lkmm.sh` 静态检查的 17 份判别力样本 |
| `selftest-lkmm.txt` | SOP 的 selftest 里以 `--static-only` 喂上面那批样本的那一段，原样摘出 |
| `gate-lkmm-and-qemu.txt` | SOP 的 gate.sh 里 LKMM 阶段与未实现清单里两个 QEMU 键，原样摘出，行号是原文件的 |
| `templates/litmus/` | install.sh 给新项目铺的一对 litmus 模板 |
| `rules/machine-first.md` | 「前提二」里 herd7 / LKMM 那张表、对照组与 `singlefs-models` 两段，以及「前提失效时收回哪几条」表里 litmus 那一行 |
| `rules/show-me-test.md` | 「最终判据是 QEMU/KVM 压测」一节，以及「门禁能证明什么」表里 litmus 声明那一行 |
| `rules/command-safety.md` | 「QEMU 虚机必须把 pid 写进文件」「测试镜像一律放临时目录」「破坏性操作先看清楚再动」三节 |
| `skills/crash-test/SKILL.md` | 「LKMM」「QEMU」两节 |

规则与 skill 是整份原样拷来的，上表写的是其中与 QEMU / herd7 有关的段落，其余段落 SOP 还留着。

## SOP 删掉这些之后要改的几处

| 哪一处 | 为什么要改 | 状态（2026-09-18 现查） |
|---|---|---|
| `.claude/gate.d/55-qemu-first-transaction.sh` 头部的 `# gate-covers: QEMU 真实负载` | SOP 未实现清单里已经没有这个键，gate.sh 会判「写了清单里没有的项」 | 已改：头部写明不声明 gate-covers 及理由 |
| `.claude/scripts/lkmm.sh` | 它原来转发到的共享脚本已删 | 已改：本项目自己维护的整份脚本，只 source SOP 的 `lib.sh` |
| `litmus/` 下六份 litmus | 共享门禁不再跑 LKMM，没有任何阶段判它们 | 已改：接成本项目的阶段 `.claude/gate.d/57-lkmm.sh` |
| `CLAUDE.md` 与 `README.md` 里跑 `bash .claude/scripts/lkmm.sh` 的那几行 | 指向的脚本曾经失效 | 不用改：`README.md` 那几行调的就是上面那份本地脚本；`CLAUDE.md` 里已经没有这一行 |
| `.claude/install-owned` 里 `litmus/commit-publish.litmus` 与 `litmus/commit-publish-nofence.litmus` 两行 | SOP 的模板里已经没有 litmus，`install.sh` 对「接管清单写了而根本不铺的路径」判红（`.claude/singlefs-ai-sop/install.sh` 第 215–219 行） | 已改（2026-09-18）：两行删掉；litmus 由 57 号阶段管，不再需要接管声明 |
