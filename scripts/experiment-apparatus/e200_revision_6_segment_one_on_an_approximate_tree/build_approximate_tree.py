"""E200（修订6近似树重跑） 造近似树：跑前登记 research/prompts/e200-preregistration.md 第一节读法 4 的 T1–T5。

造出来的（都在 --output 给的新目录下）：
  tree/            Cargo.toml、Cargo.lock、crates/ 下名字以 .rs 结尾或以 Cargo. 起头的（T1，快照摘要只收这些），加 .cargo/ 下全部与 crates/mutations.tsv
  research-side/   修订 6 那一趟研究一侧十一份（E171（三oracle合谋测试） 研究一侧 JSON 的 files）与没记 sha256 的两份，按仓里的相对路径放
  manifest.tsv     逐文件清单：place、path、class、status、pinned、marker、source、blob、sha256
blob 列（产物 name=file 行的 blob=）：内容直接取自 git 的那一版的 blob 号（T2、T3 命中的那一个（悬空的也写）、T5）；T4 删行复原的、取今天主工作区的、missing 写 -。
记号（manifest 的 marker 列，产物 name=file 行的 marker=）：
  exact_head                 T2：不在 94 行里，取 31e47115… 那一版
  exact_pinned               T3：在 94 行里、有钉值；全部提交（含 stash 与 reflog 点到的）里这条路径的某一版或某个悬空 blob，内容 sha256 等于钉值；
                             source= 写那一版出处的提交号，悬空的写 unreachable；blob= 写命中的那个 blob 号（登记第一节读法 4 T3「写 blob 号与出处提交」）
  exact_reconstructed        T4：装置 T3 找不到，ab35105f…（提交 f8a9ddaab）删第 3478–3484 行之后等于 67e7a549…
  approximate_device         T4：删完不等于 67e7a549…，照用
  approximate_first_commit   T5：31e47115… 的后代里按提交时刻最早一个、这条路径与 31e47115… 那一版不同的提交；新文件取最早加它的那一个提交
  approximate_head           T5：后代里一次都没改过，取 31e47115… 那一版；近似树乙（--variant head）里改过的也取这一版
  approximate_today          研究一侧：git 里找不到，取今天主工作区那一份（两张坏实现表除外，登记第一节读法 4 末两条）
  unrecorded                 研究一侧修订 6 那一趟没记 sha256 的两份，取今天主工作区那一份
  missing                    哪儿都没有；树里的新文件记这个走停机 S40
两种近似树（登记第八节几何敏感性）：--variant first-commit 是近似树甲；--variant head 是近似树乙（T5 一律取 31e47115… 那一版，新文件照取最早加它的那一版）。
不在树里换任何一行、不打补丁、不为编过而改（登记第一节读法 4 末条）。

跑法（仓根，整条命令交 task-server/task submit --）：
  python3 scripts/experiment-apparatus/e200_revision_6_segment_one_on_an_approximate_tree/build_approximate_tree.py \
      --repository . --variant first-commit --output <工作根下还不存在的目录> --product research/results/<还不存在的文件>
退出码：0 造完、没有停机；1 造完而停机 S40 或 S44 触发（产物照写，name=stop 行点名）；2 用法错，或输出目录、产物已存在；
  3 存档产物或研究一侧钉值表的 sha256、行数与登记不符（作废 Z42 那一类：读的不是登记点名的那几份），什么都不造。
并行（登记第十四节「独立单位」）：要算 sha256 的 blob 排好序按区间切成线程数片，每片一个 git cat-file --batch，合并按 blob 号（确定）；
  线程数取登记的每台默认重活线程数（scripts/hosts/heavy_threads.py），产物 config 行报。单线程与多线程对拍在单测里。
"""

from __future__ import annotations

import argparse
import concurrent.futures
import dataclasses
import hashlib
import json
import os
import subprocess
import sys
import threading

from result_lines import ResultLines, field

HERE = os.path.dirname(os.path.abspath(__file__))
REPOSITORY_ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

BASE_COMMIT = "31e47115471ba5502a4531d25dffcc921be36899"   # 登记 K42：第一段新树拍自这一提交加 94 行未提交改动
ARCHIVE_COMMIT = "df5866f6706a3378563c9b0aac1403992ba06ac5"   # 04ceaa904^：E171 第 1 次重跑第一段产物归档之前那一版（登记第一节读法 7）
ARCHIVED_PREFIX = "research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-"
# 登记 A42：两份冻结树产物（路径、sha256、行数）
NEW_TREE_PRODUCT = (ARCHIVED_PREFIX + "frozen-tree.out", "b720e5d28dcaead6e7c7262fd82185856e869385dad30674bf9689cb658f24c6", 125)
REVISION_6_PRODUCT = (ARCHIVED_PREFIX + "frozen-tree-r6-r3.out", "786e48010de9622d623198da5eabecba9694afd3ca73bf71cee2e3f241ca4b90", 62)
E171_RESEARCH_DIRECTORY = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
# 研究一侧十一份的钉值表：提交 f4605214d 那一版（今天主工作区那份 sha256 相同；E171 研究一侧有别人在改，读提交里的那一份）
RESEARCH_SIDE_TABLE = ("f4605214d0e671767ade774d6061f23307fc1ff8",
                       E171_RESEARCH_DIRECTORY + "/revision_8_research_side_of_the_revision_6_run.json",
                       "2fab8fad7077c41113698666673533cc928cd25ea167122a1bf4e482f718f099")
# 研究一侧没记 sha256 的两份（登记第一节读法 4 倒数第三条）
UNRECORDED_RESEARCH_FILES = (E171_RESEARCH_DIRECTORY + "/segment_one_comparator.py", E171_RESEARCH_DIRECTORY + "/s15_compare.py")
# 两张坏实现表：git 里找不到也不取今天的（今天那两份改了名、改了内容）
BAD_IMPLEMENTATION_TABLES = (E171_RESEARCH_DIRECTORY + "/bad-implementations-117.tsv",
                             E171_RESEARCH_DIRECTORY + "/bad-implementations-expansion-164.tsv")

DEVICE_PATH = "crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs"
REVISION_6_DEVICE_SHA256 = "67e7a5498425ec3f27379ba8f0e5cd666faaeaac0817006d6938956d6419a156"   # 登记 K41
NEW_TREE_DEVICE_SHA256 = "b1457bc6175fe3c2b3da50b5e3b5901c1c9a8d95ce3080fef48d2c087bbe6d84"
DEVICE_BASE_COMMIT = "f8a9ddaab885073cc311b708042cbbf94cd828a9"   # 登记 A43：git 里这条路径唯一一份修订 6 前后的版本
DEVICE_BASE_SHA256 = "ab35105f861d509457296741d2c60515a3a94b9fefe77944f03d1b358afd45df"
# 登记第一节读法 4 T4：ab35105f… 比 67e7a549… 多的一块七行（E171 第 1 次重跑修订 8 第 8.11 节第 4 条 `@@ -3475,6 +3475,13 @@`）
REMOVED_FIRST_LINE = 3478
REMOVED_LINE_COUNT = 7
REMOVED_BLOCK_NAME = "SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults"
EXPECTED_DIGEST_PATHS = 284   # 登记 K40 / A40；不是这个数走停机 S40

ADDED_STATUSES = ("A ", "AM", "??")   # 94 行里新加的文件
CLASS_DEVICE = "device"
CLASS_RESEARCH_SIDE = "research_side"
CLASS_PRODUCT = "product_code_or_pool_checker"
CLASS_OTHER = "other_verification_package"
CLASS_UNCLASSIFIED = "unclassified"
DEVICE_DIRECTORY = "crates/singlefs-checker-tier/src/bin/"
# 登记第一节读法 3 的第三类与第四类（第十二节修订 1 第 2 条：singlefs-replay-prefix 归第三类，crates/mutations.tsv 归第四类）
PRODUCT_PREFIXES = ("crates/singlefs-core/", "crates/singlefs-format/", "crates/singlefs-checker/", "crates/singlefs-replay-prefix/")
OTHER_PREFIXES = ("crates/singlefs-harness/", "crates/singlefs-checker-tier/", "crates/singlefs-reference-filesystem/")

MARKER_EXACT_HEAD = "exact_head"
MARKER_EXACT_PINNED = "exact_pinned"
MARKER_EXACT_RECONSTRUCTED = "exact_reconstructed"
MARKER_APPROXIMATE_DEVICE = "approximate_device"
MARKER_FIRST_COMMIT = "approximate_first_commit"
MARKER_APPROXIMATE_HEAD = "approximate_head"
MARKER_TODAY = "approximate_today"
MARKER_UNRECORDED = "unrecorded"
MARKER_MISSING = "missing"
UNREACHABLE = "unreachable"
VARIANT_FIRST_COMMIT = "first-commit"
VARIANT_HEAD = "head"


def classify(path: str) -> str:
    """登记第一节读法 3 的四类：装置（bin/ 下全部）、研究一侧、产品代码与池级 checker、别的验证包（Cargo.lock、crates/mutations.tsv 与各 Cargo.toml 算这一类）。"""
    if path.startswith(DEVICE_DIRECTORY):
        return CLASS_DEVICE
    if path.startswith("scripts/experiment-apparatus/"):
        return CLASS_RESEARCH_SIDE
    if path in ("Cargo.lock", "crates/mutations.tsv") or os.path.basename(path) == "Cargo.toml":
        return CLASS_OTHER
    if path.startswith(PRODUCT_PREFIXES):
        return CLASS_PRODUCT
    if path.startswith(OTHER_PREFIXES):
        return CLASS_OTHER
    return CLASS_UNCLASSIFIED


def counted_in_digest(path: str) -> bool:
    """T1：根上的 Cargo.toml、Cargo.lock 与 crates/ 下（不进 target/）名字以 .rs 结尾或以 Cargo. 起头的（同快照摘要的收法）。"""
    name = os.path.basename(path)
    if path in ("Cargo.toml", "Cargo.lock"):
        return True
    return path.startswith("crates/") and "/target/" not in path and (name.endswith(".rs") or name.startswith("Cargo."))


def parse_status_line(line: str) -> tuple[str, str]:
    """冻结树产物里一行 key=git_status 的 line= 值：两位状态（空格写成 _）、一个 _、路径。交回 (状态, 路径)。"""
    return line[:2].replace("_", " "), line[3:]


@dataclasses.dataclass(frozen=True)
class Version:
    """31e47115… 的一个后代提交上这条路径的那一版：blob 为 None 表示那个提交里没有这条路径；parent_blobs 是各父提交里这条路径的 blob。"""
    time: int
    commit: str
    blob: str | None
    parent_blobs: tuple[str | None, ...]


@dataclasses.dataclass(frozen=True)
class Pin:
    """一份钉值：路径、出自哪一份（new_tree_file：新树产物的 key=file_sha256；revision_6_tree：修订 6 那一趟第 0 步产物的 tree=）、目标 sha256。"""
    path: str
    role: str
    target_sha256: str


@dataclasses.dataclass(frozen=True)
class Choice:
    """一条路径取的那一版：记号、出处（提交号、unreachable、today、-）、blob 号（内容直接取自 git 的那一个；复原的、今天的、missing 是 -）、内容（None 是 missing）。"""
    marker: str
    source: str
    blob: str
    content: bytes | None


class DeviceBlockMismatch(Exception):
    """停机 S44：ab35105f… 第 3478–3484 行里没有登记点名的那个枚举名，删的不是那一块。"""


def t3_candidates(path_versions: dict[str, str], unreachable_blobs: list[str]) -> dict[str, str]:
    """T3 的候选 blob → 出处：这条路径在全部提交（含 stash 与 reflog）里的每一版（出处是提交号），加每一个悬空 blob（出处 unreachable）。"""
    candidates = dict(path_versions)
    for blob in unreachable_blobs:
        candidates.setdefault(blob, UNREACHABLE)
    return candidates


def find_pinned(candidates: dict[str, str], sha256_of_blob: dict[str, str], target_sha256: str) -> tuple[str, str] | None:
    """候选里内容 sha256 等于钉值的那一个 → (blob, 出处)；几个都等时先取有提交出处的，再按 blob 号（确定）。找不到交 None。"""
    found = sorted((origin == UNREACHABLE, blob, origin) for blob, origin in candidates.items() if sha256_of_blob.get(blob) == target_sha256)
    return (found[0][1], found[0][2]) if found else None


def first_change_after_base(versions: list[Version], base_blob: str) -> Version | None:
    """T5（31e47115… 里有的路径）：后代里按提交时刻最早一个、这条路径在、且与 31e47115… 那一版不同的提交；一次都没改过交 None。"""
    changed = [version for version in versions if version.blob is not None and version.blob != base_blob]
    return min(changed, key=lambda version: (version.time, version.commit)) if changed else None


def first_addition_after_base(versions: list[Version]) -> Version | None:
    """T5（新文件）：后代里按提交时刻最早一个加它的提交（这个提交里有这条路径，它的父提交里都没有）；从没加过交 None。"""
    additions = [version for version in versions if version.blob is not None and all(parent is None for parent in version.parent_blobs)]
    return min(additions, key=lambda version: (version.time, version.commit)) if additions else None


def remove_device_block(text: bytes) -> bytes:
    """T4：删掉第 REMOVED_FIRST_LINE 行起的 REMOVED_LINE_COUNT 行（行号从 1 起）；删之前核这几行里有 REMOVED_BLOCK_NAME，没有抛 DeviceBlockMismatch。"""
    lines = text.splitlines(keepends=True)
    start = REMOVED_FIRST_LINE - 1
    block = b"".join(lines[start:start + REMOVED_LINE_COUNT])
    if REMOVED_BLOCK_NAME.encode("utf-8") not in block:
        raise DeviceBlockMismatch(f"第 {REMOVED_FIRST_LINE}–{REMOVED_FIRST_LINE + REMOVED_LINE_COUNT - 1} 行里没有 {REMOVED_BLOCK_NAME}")
    return b"".join(lines[:start] + lines[start + REMOVED_LINE_COUNT:])


def sha256_hex(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def snapshot_digest(root: str) -> tuple[str, int]:
    """快照摘要：照抄 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py 的 snapshot_digest
    （提交 59ce5a865 那一版，文件 sha256 dfaeb9edf59694b3b63c790e170573a1522fe07778d2fd179876ab7ef70821e1）：root 下每个 .rs 与 Cargo.* 文件的
    (相对路径, sha256) 排好序，`<路径> <sha256>` 用换行连起来再算一个 sha256，连同文件数。"""
    entries = []
    for directory, _, files in os.walk(root):
        for file_name in files:
            if file_name.endswith(".rs") or file_name.startswith("Cargo."):
                path = os.path.join(directory, file_name)
                with open(path, "rb") as handle:
                    entries.append((os.path.relpath(path, root), hashlib.sha256(handle.read()).hexdigest()))
    entries.sort()
    return hashlib.sha256("\n".join(f"{path} {digest}" for path, digest in entries).encode("utf-8")).hexdigest(), len(entries)


def digest_path_set(base_paths: list[str], status_entries: list[tuple[str, str]]) -> tuple[list[str], list[str]]:
    """T1：交回 (31e47115… 里进快照摘要的路径, 94 行里新加的进快照摘要的路径)，各自排好序。"""
    in_base = sorted(path for path in base_paths if counted_in_digest(path))
    added = sorted(path for status, path in status_entries if status in ADDED_STATUSES and counted_in_digest(path))
    return in_base, added


def extra_paths(base_paths: list[str]) -> list[str]:
    """T1 另带的（不进快照摘要，进逐文件表）：.cargo/ 下全部与 crates/mutations.tsv。"""
    return sorted(path for path in base_paths if path.startswith(".cargo/") or path == "crates/mutations.tsv")


class GitRepository:
    """读 git 的那一层（单测换成临时仓）：只读，不改引用、不改工作区、不 gc。"""

    def __init__(self, root: str):
        self.root = root
        self._blobs_cache: dict[str, dict[str, str]] = {}
        self.fsck_exit = -1

    def git(self, *arguments: str) -> bytes:
        return subprocess.run(["git", "-C", self.root, *arguments], check=True, capture_output=True).stdout

    def tree_paths(self, commit: str) -> list[str]:
        return sorted(name for name in self.git("ls-tree", "-r", "-z", "--name-only", commit).decode("utf-8").split("\0") if name)

    def blobs_at(self, commit: str, paths: tuple[str, ...]) -> dict[str, str]:
        """一个提交里这几条路径的 blob（不在的不出现）；按提交缓存。"""
        key = commit + "\0" + "\0".join(paths)
        if key not in self._blobs_cache:
            found = {}
            for entry in self.git("ls-tree", "-r", "-z", commit, "--", *paths).decode("utf-8").split("\0"):
                if entry:
                    header, path = entry.split("\t", 1)
                    if header.split()[1] == "blob":
                        found[path] = header.split()[2]
            self._blobs_cache[key] = found
        return self._blobs_cache[key]

    def blob_at(self, commit: str, path: str) -> str | None:
        return self.blobs_at(commit, (path,)).get(path)

    def read_blob(self, blob: str) -> bytes:
        return self.git("cat-file", "blob", blob)

    def descendants(self, base: str) -> list[tuple[int, str, tuple[str, ...]]]:
        """base 的全部后代（全部引用与 reflog 点到的提交里）：(提交时刻, 提交号, 父提交号)。"""
        lines = self.git("log", "--all", "--reflog", "--ancestry-path", "^" + base, "--format=%ct %H %P").decode("utf-8").splitlines()
        found = {}
        for line in lines:
            parts = line.split()
            found[parts[1]] = (int(parts[0]), parts[1], tuple(parts[2:]))
        return sorted(found.values())

    def versions_after(self, base: str, path: str, descendants: list[tuple[int, str, tuple[str, ...]]], paths: tuple[str, ...]) -> list[Version]:
        """这条路径在 base 每个后代上的那一版（父提交里的那一版一起带上，T5 认「加它的提交」用）。"""
        return [Version(time, commit, self.blobs_at(commit, paths).get(path),
                        tuple(self.blobs_at(parent, paths).get(path) for parent in parents))
                for time, commit, parents in descendants]

    def path_versions(self, path: str) -> dict[str, str]:
        """这条路径在全部提交（--all 加 --reflog：分支、标签、stash 与 reflog 点到的，连同它们的祖先）里出现过的每一版 → 最早出它的那个提交。"""
        output = self.git("log", "--all", "--reflog", "--full-history", "-m", "--root", "--raw", "--no-abbrev", "--no-renames",
                          "--format=commit %H", "--", path).decode("utf-8")
        found: dict[str, str] = {}
        commit = ""
        for line in output.splitlines():
            if line.startswith("commit "):
                commit = line.split()[1]
            elif line.startswith(":"):
                for blob in line.split("\t")[0].split()[2:4]:
                    if set(blob) != {"0"}:
                        found[blob] = commit   # log 从新到旧，留下的是最早那一个
        return found

    def unreachable_blobs(self) -> list[str]:
        """git fsck --unreachable --no-reflogs 列的每一个悬空 blob（只被 reflog 点到的也算进来）。fsck 报了别的毛病退非 0 时照收它列出的，
        退出码记在 self.fsck_exit（产物 name=search 行报）。"""
        completed = subprocess.run(["git", "-C", self.root, "fsck", "--unreachable", "--no-reflogs"], capture_output=True)
        self.fsck_exit = completed.returncode
        return sorted(line.split()[2] for line in completed.stdout.decode("utf-8", errors="replace").splitlines()
                      if line.startswith("unreachable blob "))

    def sha256_of_blobs(self, blobs: list[str]) -> dict[str, str]:
        """一个 git cat-file --batch 读这一片 blob，边读边算 sha256（不把整片内容留在内存里）；读不到的不出现。"""
        process = subprocess.Popen(["git", "-C", self.root, "cat-file", "--batch"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        writer = threading.Thread(target=lambda: (process.stdin.write("".join(blob + "\n" for blob in blobs).encode("ascii")), process.stdin.close()))
        writer.start()
        found = {}
        for blob in blobs:
            header = process.stdout.readline().decode("ascii").split()
            if len(header) != 3:
                continue
            remaining, hasher = int(header[2]), hashlib.sha256()
            while remaining:
                chunk = process.stdout.read(min(remaining, 1 << 20))
                hasher.update(chunk)
                remaining -= len(chunk)
            process.stdout.read(1)
            found[blob] = hasher.hexdigest()
        writer.join()
        process.stdout.close()
        if process.wait() != 0:
            raise RuntimeError("git cat-file --batch 退出非 0")
        return found


def hash_blobs(repository: GitRepository, blobs: list[str], threads: int) -> dict[str, str]:
    """blob → 内容 sha256：排好序按区间切成 threads 片，各片一个 git cat-file --batch 并行读，合并按片的次序（确定）。"""
    ordered = sorted(set(blobs))
    if not ordered:
        return {}
    size = -(-len(ordered) // max(1, threads))
    slices = [ordered[start:start + size] for start in range(0, len(ordered), size)]
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(slices)) as pool:
        results = list(pool.map(repository.sha256_of_blobs, slices))
    merged: dict[str, str] = {}
    for part in results:
        merged.update(part)
    return merged


class ArchivedInputMismatch(Exception):
    """存档产物或研究一侧钉值表与登记点名的不是同一份（退 3）。"""


def key_values(line: str) -> dict[str, str]:
    return dict(part.split("=", 1) for part in line.split() if "=" in part)


def archived_lines(repository: GitRepository, product: tuple[str, str, int]) -> list[str]:
    """从 ARCHIVE_COMMIT 取一份存档产物，核 sha256 与行数等于登记 A42 写的（不等抛 ArchivedInputMismatch）。"""
    path, expected_sha256, expected_lines = product
    content = repository.git("show", f"{ARCHIVE_COMMIT}:{path}")
    lines = content.decode("utf-8").splitlines()
    if sha256_hex(content) != expected_sha256 or len(lines) != expected_lines:
        raise ArchivedInputMismatch(f"{path}：sha256 {sha256_hex(content)}、{len(lines)} 行，登记是 {expected_sha256}、{expected_lines} 行")
    return lines


def load_status_entries(repository: GitRepository) -> list[tuple[str, str]]:
    """新树产物里 94 行 key=git_status 的 (状态, 路径)。"""
    lines = archived_lines(repository, NEW_TREE_PRODUCT)
    return [parse_status_line(key_values(line)["line"]) for line in lines if " key=git_status " in line]


def load_pins(repository: GitRepository) -> list[Pin]:
    """登记第一节读法 2 的十五份钉值：新树产物 13 行 key=file_sha256（装置那一行的目标换成修订 6 树那一份 67e7a549…），
    加修订 6 那一趟第 0 步产物 2 行 key=file_sha256 的 tree=。装置那一份核两份产物与登记 K41 一致。"""
    pins = []
    for line in archived_lines(repository, NEW_TREE_PRODUCT):
        if " key=file_sha256 " in line:
            values = key_values(line)
            if values["file"] == DEVICE_PATH:
                if values["sha256"] != NEW_TREE_DEVICE_SHA256:
                    raise ArchivedInputMismatch(f"新树产物里装置 {values['sha256']}，登记是 {NEW_TREE_DEVICE_SHA256}")
                pins.append(Pin(DEVICE_PATH, "new_tree_file", REVISION_6_DEVICE_SHA256))
            else:
                pins.append(Pin(values["file"], "new_tree_file", values["sha256"]))
    revision_6_lines = archived_lines(repository, REVISION_6_PRODUCT)
    device_in_tree = [key_values(line)["sha256"] for line in revision_6_lines if " key=device_sha256 place=tree " in line]
    if device_in_tree != [REVISION_6_DEVICE_SHA256]:
        raise ArchivedInputMismatch(f"修订 6 那一趟第 0 步产物的 device_sha256 place=tree 是 {device_in_tree}，登记是 {REVISION_6_DEVICE_SHA256}")
    for line in revision_6_lines:
        if " key=file_sha256 " in line:
            values = key_values(line)
            pins.append(Pin(values["file"], "revision_6_tree", values["tree"]))
    return pins


def research_side_pins(repository: GitRepository) -> list[tuple[str, str, bool]]:
    """研究一侧十一份：(仓里的相对路径, 钉值, 钉值表里记的是不是仓外草稿目录里那一份)。仓外那一份按文件名对到 E171 研究一侧目录，产物只写仓里的路径。"""
    commit, path, expected = RESEARCH_SIDE_TABLE
    content = repository.git("show", f"{commit}:{path}")
    if sha256_hex(content) != expected:
        raise ArchivedInputMismatch(f"{path}@{commit[:9]}：sha256 {sha256_hex(content)}，登记是 {expected}")
    pins = []
    for key, value in json.loads(content)["files"].items():
        outside = not key.startswith("scripts/")
        pins.append((E171_RESEARCH_DIRECTORY + "/" + os.path.basename(key) if outside else key, value, outside))
    return sorted(pins)


def choose_tree_version(repository: GitRepository, path: str, status: str | None, pin: Pin | None, variant: str,
                        candidates_of, sha256_of_blob: dict[str, str], versions: list[Version]) -> tuple[Choice, dict[str, str]]:
    """一条树里路径照 T2–T5 取哪一版。交回 (取的那一版, 装置那一份的附带读数)。candidates_of(path) 交 T3 的候选。"""
    notes: dict[str, str] = {}
    if status is None:
        t2_blob = repository.blob_at(BASE_COMMIT, path)
        return Choice(MARKER_EXACT_HEAD, BASE_COMMIT, t2_blob, repository.read_blob(t2_blob)), notes
    if pin is not None:
        found = find_pinned(candidates_of(path), sha256_of_blob, pin.target_sha256)
        if path == DEVICE_PATH:
            notes["t3"] = "found" if found else "not_found"
        if found:
            return Choice(MARKER_EXACT_PINNED, found[1], found[0], repository.read_blob(found[0])), notes
        if path == DEVICE_PATH:
            device_base = repository.read_blob(repository.blob_at(DEVICE_BASE_COMMIT, path))
            notes["device_base_sha256"] = sha256_hex(device_base)
            try:
                rebuilt = remove_device_block(device_base)
            except DeviceBlockMismatch as problem:
                notes["s44"] = str(problem)
            else:
                exact = sha256_hex(rebuilt) == REVISION_6_DEVICE_SHA256
                return Choice(MARKER_EXACT_RECONSTRUCTED if exact else MARKER_APPROXIMATE_DEVICE, DEVICE_BASE_COMMIT, "-", rebuilt), notes
    base_blob = repository.blob_at(BASE_COMMIT, path)
    if base_blob is None:
        added = first_addition_after_base(versions)
        if added is None:
            return Choice(MARKER_MISSING, "-", "-", None), notes
        return Choice(MARKER_FIRST_COMMIT, added.commit, added.blob, repository.read_blob(added.blob)), notes
    changed = first_change_after_base(versions, base_blob) if variant == VARIANT_FIRST_COMMIT else None
    if changed is None:
        return Choice(MARKER_APPROXIMATE_HEAD, BASE_COMMIT, base_blob, repository.read_blob(base_blob)), notes
    return Choice(MARKER_FIRST_COMMIT, changed.commit, changed.blob, repository.read_blob(changed.blob)), notes


def choose_research_version(path: str, pinned: str | None, candidates: dict[str, str], sha256_of_blob: dict[str, str],
                            repository: GitRepository, today_root: str) -> Choice:
    """研究一侧一份：有钉值的照 T3 找；找不到取今天主工作区那一份（两张坏实现表不取，记 missing）；没钉值的取今天那一份、记 unrecorded。"""
    today = os.path.join(today_root, path)
    if pinned is not None:
        found = find_pinned(candidates, sha256_of_blob, pinned)
        if found:
            return Choice(MARKER_EXACT_PINNED, found[1], found[0], repository.read_blob(found[0]))
        if path in BAD_IMPLEMENTATION_TABLES or not os.path.isfile(today):
            return Choice(MARKER_MISSING, "-", "-", None)
        with open(today, "rb") as handle:
            return Choice(MARKER_TODAY, "today", "-", handle.read())
    if not os.path.isfile(today):
        return Choice(MARKER_MISSING, "-", "-", None)
    with open(today, "rb") as handle:
        return Choice(MARKER_UNRECORDED, "today", "-", handle.read())


@dataclasses.dataclass
class BuildInputs:
    """造树的输入（命令行从存档产物与钉值表读；单测手造）。"""
    status_entries: list[tuple[str, str]]
    pins: list[Pin]
    research_pins: list[tuple[str, str, bool]]
    expected_digest_paths: int = EXPECTED_DIGEST_PATHS


def write_file(root: str, path: str, content: bytes) -> None:
    target = os.path.join(root, path)
    os.makedirs(os.path.dirname(target), exist_ok=True)
    with open(target, "xb") as handle:
        handle.write(content)


def build(repository: GitRepository, inputs: BuildInputs, variant: str, output: str, lines: ResultLines, threads: int,
          today_root: str) -> int:
    """照 T1–T5 造一棵近似树进 output（output 不许已存在）。交回退出码：0 没有停机，1 停机 S40 或 S44。"""
    status_of = {path: status for status, path in inputs.status_entries}
    pin_of = {pin.path: pin for pin in inputs.pins}
    base_paths = repository.tree_paths(BASE_COMMIT)
    in_base, added = digest_path_set(base_paths, inputs.status_entries)
    overlap = sorted(set(in_base) & set(added))
    tree_paths = sorted(set(in_base) | set(added)) + extra_paths(base_paths)
    stops = []
    lines.result(f"name=path_set base_paths={len(in_base)} added_paths={len(added)} overlap={len(overlap)} digest_paths={len(in_base) + len(added) - len(overlap)} "
                 f"expected={inputs.expected_digest_paths} extra_paths={len(tree_paths) - len(set(in_base) | set(added))} "
                 f"holds={str(len(set(in_base) | set(added)) == inputs.expected_digest_paths and not overlap).lower()}")
    if len(set(in_base) | set(added)) != inputs.expected_digest_paths or overlap:
        stops.append(("S40", f"path_set_{len(set(in_base) | set(added))}_expected_{inputs.expected_digest_paths}_overlap_{len(overlap)}"))
    pinned_in_status = sorted(path for path in pin_of if path in status_of)
    research_paths = [path for path, _, _ in inputs.research_pins]
    path_versions = {path: repository.path_versions(path) for path in pinned_in_status + research_paths}
    unreachable = repository.unreachable_blobs()
    to_hash = set(unreachable)
    for versions in path_versions.values():
        to_hash.update(versions)
    sha256_of_blob = hash_blobs(repository, sorted(to_hash), threads)
    lines.result(f"name=search unreachable_blobs={len(unreachable)} hashed_blobs={len(sha256_of_blob)} pinned_paths_in_status={len(pinned_in_status)} "
                 f"research_side_paths={len(research_paths)} fsck_exit={repository.fsck_exit}")
    descendants = repository.descendants(BASE_COMMIT)
    changed_paths = tuple(sorted(path for path in tree_paths if path in status_of))
    tree_root, manifest_rows = os.path.join(output, "tree"), []
    for path in tree_paths:
        versions = repository.versions_after(BASE_COMMIT, path, descendants, changed_paths) if path in status_of else []
        choice, notes = choose_tree_version(repository, path, status_of.get(path), pin_of.get(path), variant,
                                            lambda pinned_path: t3_candidates(path_versions[pinned_path], unreachable), sha256_of_blob, versions)
        digest = sha256_hex(choice.content) if choice.content is not None else "-"
        if choice.content is not None:
            write_file(tree_root, path, choice.content)
        elif counted_in_digest(path):
            stops.append(("S40", f"new_file_missing_{path}"))
        row = ("tree", path, classify(path), (status_of.get(path) or "none").replace(" ", "_"), str(path in pin_of).lower(), choice.marker, choice.source,
               choice.blob, digest)
        manifest_rows.append(row)
        lines.result("name=file place=tree " + " ".join(f"{key}={field(value)}" for key, value in
                                                         zip(("path", "class", "status", "pinned", "marker", "source", "blob", "sha256"), row[1:])))
        if path == DEVICE_PATH:
            if "s44" in notes:
                stops.append(("S44", notes["s44"]))
            lines.result(f"name=device t3={notes.get('t3', '-')} device_base_sha256={notes.get('device_base_sha256', '-')} "
                         f"removed_lines={REMOVED_FIRST_LINE}-{REMOVED_FIRST_LINE + REMOVED_LINE_COUNT - 1} block_name_found={str('s44' not in notes).lower()} "
                         f"marker={choice.marker} sha256={digest} equals_revision_6_device={str(digest == REVISION_6_DEVICE_SHA256).lower()} "
                         f"equals_new_tree_device={str(digest == NEW_TREE_DEVICE_SHA256).lower()}")
    research_root = os.path.join(output, "research-side")
    research_entries = [(path, pinned, outside) for path, pinned, outside in inputs.research_pins] + \
                       [(path, None, False) for path in UNRECORDED_RESEARCH_FILES]
    for path, pinned, outside in research_entries:
        candidates = t3_candidates(path_versions.get(path, {}), unreachable) if pinned is not None else {}
        choice = choose_research_version(path, pinned, candidates, sha256_of_blob, repository, today_root)
        digest = sha256_hex(choice.content) if choice.content is not None else "-"
        if choice.content is not None:
            write_file(research_root, path, choice.content)
        manifest_rows.append(("research_side", path, classify(path), "none", str(pinned is not None).lower(), choice.marker, choice.source, choice.blob,
                              digest))
        lines.result(f"name=file place=research_side path={field(path)} pinned_outside_the_repository={str(outside).lower()} "
                     f"marker={choice.marker} source={field(choice.source)} blob={choice.blob} sha256={digest}")
    with open(os.path.join(output, "manifest.tsv"), "x", encoding="utf-8") as handle:
        handle.write("place\tpath\tclass\tstatus\tpinned\tmarker\tsource\tblob\tsha256\n")
        handle.writelines("\t".join(row) + "\n" for row in manifest_rows)
    digest, count = snapshot_digest(tree_root)
    markers: dict[str, int] = {}
    for row in manifest_rows:
        markers[f"{row[0]}:{row[5]}"] = markers.get(f"{row[0]}:{row[5]}", 0) + 1
    lines.result(f"name=summary snapshot_digest={digest} digest_files={count} manifest_rows={len(manifest_rows)} "
                 f"manifest_sha256={sha256_of_file(os.path.join(output, 'manifest.tsv'))} "
                 f"markers={','.join(f'{key}:{value}' for key, value in sorted(markers.items()))}")
    for clause, reason in stops:
        lines.result(f"name=stop clause={clause} reason={field(reason)}")
    lines.result(f"name=verdict stops={len(stops)} built={str(not any(clause == 'S40' for clause, _ in stops)).lower()}")
    return 1 if stops else 0


def sha256_of_file(path: str) -> str:
    with open(path, "rb") as handle:
        return sha256_hex(handle.read())


def registered_threads(repository_root: str) -> int:
    """登记的每台默认重活线程数（.claude/heavy-thread-default，经 scripts/hosts/heavy_threads.py 取；SINGLEFS_THREAD_CAP 设了照它）。"""
    sys.path.insert(0, os.path.join(repository_root, "scripts", "hosts"))
    import heavy_threads
    return heavy_threads.default_heavy_threads()


def home_relative(path: str) -> str:
    """产物里写路径时家目录写成 ~（不把本机的家目录写进仓）。"""
    home = os.path.expanduser("~")
    absolute = os.path.abspath(path)
    return "~" + absolute[len(home):] if absolute == home or absolute.startswith(home + os.sep) else absolute


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E200 造近似树（登记第一节读法 4 的 T1–T5）")
    parser.add_argument("--repository", required=True, help="仓根（读 git 与今天主工作区里研究一侧那几份）")
    parser.add_argument("--variant", required=True, choices=(VARIANT_FIRST_COMMIT, VARIANT_HEAD))
    parser.add_argument("--output", required=True, help="还不存在的目录：近似树、研究一侧与逐文件清单")
    parser.add_argument("--product", required=True, help="还不存在的产物文件（research/results/ 下）")
    arguments = parser.parse_args(argument_list)
    if os.path.exists(arguments.output) or os.path.exists(arguments.product):
        print(f"输出目录 {arguments.output} 或产物 {arguments.product} 已存在：不覆盖。要重造，换一个新名字", file=sys.stderr)
        return 2
    repository = GitRepository(os.path.abspath(arguments.repository))
    try:
        inputs = BuildInputs(load_status_entries(repository), load_pins(repository), research_side_pins(repository))
    except ArchivedInputMismatch as problem:
        print(f"作废 Z42 那一类：{problem}；什么都没造", file=sys.stderr)
        return 3
    threads = registered_threads(REPOSITORY_ROOT)
    lines = ResultLines(arguments.product)
    lines.result(f"name=config variant={arguments.variant} base_commit={BASE_COMMIT} archive_commit={ARCHIVE_COMMIT} threads={threads} "
                 f"output={field(home_relative(arguments.output))} status_lines={len(inputs.status_entries)} pins={len(inputs.pins)} "
                 f"research_side_pins={len(inputs.research_pins)}")
    for role, product in (("new_tree", NEW_TREE_PRODUCT), ("revision_6_run", REVISION_6_PRODUCT)):
        lines.result(f"name=archived_product role={role} path={product[0]} sha256={product[1]} lines={product[2]} holds=true")
    lines.result(f"name=research_side_table commit={RESEARCH_SIDE_TABLE[0]} path={RESEARCH_SIDE_TABLE[1]} sha256={RESEARCH_SIDE_TABLE[2]} holds=true")
    os.makedirs(arguments.output)
    exit_code = build(repository, inputs, arguments.variant, arguments.output, lines, threads, repository.root)
    lines.finish()
    return exit_code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
