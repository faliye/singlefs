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

## singlefs 下次同步 SOP 之前要改的

SOP 删掉这些之后，照旧同步会在这几处红或者失效：

- `.claude/gate.d/55-qemu-first-transaction.sh` 头部的 `# gate-covers: QEMU 真实负载`：SOP 未实现清单里已经没有这个键，gate.sh 会判「写了清单里没有的项」。
- `.claude/scripts/lkmm.sh`：它转发到的共享脚本已删，跑它只会报「找不到共享脚本」。
- `.claude/install-owned` 里 `litmus/commit-publish.litmus` 与 `litmus/commit-publish-nofence.litmus` 两行：SOP 不再铺 litmus，install.sh 会判「接管清单写了不铺的路径」。
- `litmus/` 下六份 litmus 从此没有任何阶段判：共享门禁不再跑 LKMM。要继续判，就把上面的 `lkmm.sh` 接成本项目自己的阶段。
- `CLAUDE.md` 与 `README.md` 里跑 `bash .claude/scripts/lkmm.sh` 的那几行。
