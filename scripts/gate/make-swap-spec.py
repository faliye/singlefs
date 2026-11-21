#!/usr/bin/env python3
# admission: always 每一次都现比此刻的基线与新版两棵树，上一次生成的规格不替这一次作保
# run-condition: command python3 rsync
# gate-similar: replace-batch.py 它照一份写好的规格在仓里定点改文件（施加方），这里从基线与新版算出规格、一个仓内文件都不改（生成方）；并进去等于让改文件的工具也管生成，两种写权限混在一份里
# gate-similar: hook-swap.py 它拿规格先验后换、换上之后再验与回滚，要起钩子、跑验证命令、写主工作区，只由主 agent 经任务服务起；这里是纯文本、只写工作根下的 --out，工具员在草稿目录里就要用
"""从基线与新版算出 scripts/gate/hook-swap.py 的规格：每处改动一项，正着依次施加、倒着依次施加都恰好命中一次，两头核过才写出。

用法：
  make-swap-spec.py --base <基线根> --new <新版根> --out <规格.json> [--max-context <行数>] <相对路径>…
  make-swap-spec.py --selftest        # MAKE_SWAP_SPEC_BREAK=<项> 时必须判红

<基线根> 一般是主工作区（仓根）或草稿目录里照主工作区此刻那一份拷的基线树，<新版根> 是草稿目录里写好新版的那棵树；<相对路径> 是要换的文件相对两个根的路径，
  写进规格的 file 也是它（hook-swap.py 照 --root 拼）。每份都要在两边各是一个 UTF-8 的普通文件；同一份点两次（归一之后相同也算）、绝对路径、归一之后落到根外（以 .. 起头）的拒。
<规格.json> 是 hook-swap.py 文件头「<规格.json>」那一段的 JSON 数组（每项 file / old / new，count 省掉即 1），同一份文件的几处按数组次序依次施加。
  --out 要落在工作根下（根本身不收），它所在的目录要已经在、它自己不是目录；已有的同名文件整份换掉（同目录里排他新建、名字带随机段的临时文件，写完再改名；不跟随符号链接）。
  工作根只从多机配置取（.claude/hooks/lib_work_root.py 的 shared_data_path("work")，在空环境里取：不看命令前的 SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT 与 SINGLEFS_MULTI_HOST_CONFIG），
  配置是这份工具所在仓（主工作区、git worktree 或仓副本）的多机配置；环境里设着工作根库的弄坏开关 SINGLEFS_WORK_ROOT_BREAK 的也拒（平常跑在读参数之前就退 2，点名它只在自证里认）；取不到拒；不认 SINGLEFS_MULTI_HOST_CONFIG。
  不认「这份工具在不在仓副本里」：副本是谁都改得了的一棵树，副本根上手写的多机配置定出的工作根也照认。守的是 --out 落在哪：--out 所在目录的真实路径（逐段解开符号链接之后）
  与 --out 自己的真实路径，往上任何一层有 .git（目录或文件，工作树的顶）就拒、退 3，规格要写在草稿目录里不在任何 git 工作树内的位置；符号链接指进工作树的同样拒。
  仓副本里没有多机配置的，工作根取不到，出路是起主工作区那一份：python3 <主工作区>/scripts/gate/make-swap-spec.py --base <基线根> --new <新版根> --out <规格.json> …
  （它只读 --base、--new 两棵树，不要求在副本里起）。检出钩子 ⑫ 把这一份的 --out 当定点写脚本的目标（bash-command-detector.sh 的 KNOWN_WRITE_SCRIPTS）。
  两边逐字相同的文件不出项、成功句里点名；点名的文件全都逐字相同（一处改动都没有）的拒。

做法（每份文件各算各的）：
  1. 按行比基线与新版（difflib.SequenceMatcher，不丢「垃圾行」），每个不相同的块一项。
  2. 每一项从只带改动行本身起，前后各多带一行上下文，直到同时满足三条，最多带 --max-context 行（默认 MAX_CONTEXT_LINES）：
     旧串、新串都不空；旧串在施加到这一步时的文本里恰好命中 1 次；新串在这一步施加之后的文本里恰好命中 1 次（倒着施加到这一步时定得了位），
     并且在整份规格施加完之后的文本里也恰好命中 1 次。带到上限还有一条不满足就拒，点名那一处（基线第几行起、哪一条不满足），不写规格。
     上下文行在前后几项之间可以重叠：每一项都按前面几项施加之后的文本取。
  3. 写出之前整份核（与 hook-swap.py 的 apply_edits 同一种施加：每处数命中次数、不是 1 就停）：正着依次施加等于新版、倒着依次施加（新旧对调、次序倒排）回到基线；
     核不过退 1、不写。
只读基线与新版，只写 --out；读多机配置与 .claude/hooks/lib_work_root.py 取工作根；出规格不起子进程（自证搭假仓经 scripts/hosts/lib_repository_copy.py 的 copy_entries 起 rsync，所以运行条件要有 rsync）。

退出码：0 写出了；1 有一处带到上限还定不了位，或整份核不过（都不写规格）；2 用法错（缺参数、不认的选项、--max-context 不是正整数、同一份点两次、路径绝对或落到根外，
  弄坏开关设在自证之外或值不认识）；3 输入不对（点名的文件在基线或新版里没有、不是普通文件、不是 UTF-8，点名的文件全都逐字相同，
  --out 不在工作根下、是目录或所在的目录不在、落在 git 工作树里，工作根取不到）。

弄坏开关（只给自证判红用；平常跑设了退 2、点名只在自证里认）。一条一行「MAKE_SWAP_SPEC_BREAK=<项>  弄坏哪一处」，下一行起写打开后自证红的格（失败行 `✗ make-swap-spec 自证：` 之后的格名）：
  MAKE_SWAP_SPEC_BREAK=reverse-check-skipped  只核正着施加：定位时不看新串命中几次，写出之前也不做整份核（倒着施加回不到基线照写）
      「新串在施加之后出现两次的那一形：自动多带上下文，正着倒着都核过」
  MAKE_SWAP_SPEC_BREAK=context-cap-ignored  带到上限还定不了位也不拒，交出带满上限的那一项（之后整份核不过、退 1，拒绝说明不点名那一处）
      「带到上限仍定不了位的拒：点名那一处，不写规格」
  MAKE_SWAP_SPEC_BREAK=missing-base-file-accepted  基线里没有的文件当空文件比
      「基线里没有点名的文件的拒：点名它，不写规格」
  MAKE_SWAP_SPEC_BREAK=out-outside-work-root-accepted  --out 不核在不在工作根下
      「--out 不在工作根下的拒：不写」「工作根不认命令前的环境变量：SINGLEFS_WORK_ROOT 指到别处的 --out 拒，照多机配置的工作根写」
      「仓副本里起不认副本：副本根上手写的多机配置定出的工作根下照常出规格，工作根之外的拒」
  MAKE_SWAP_SPEC_BREAK=empty-spec-accepted  点名的文件全都逐字相同也照写（写出一份空数组）
      「基线与新版逐字相同、一处改动都没有的拒：不写」
  MAKE_SWAP_SPEC_BREAK=non-utf8-decoded-leniently  不是 UTF-8 的文件按「替换字符」解、照常出规格
      「新版不是 UTF-8 文本的拒：点名它，不写规格」
  MAKE_SWAP_SPEC_BREAK=out-not-file-accepted  --out 是目录、所在的目录不在也往下写（写的时候才抛错）
      「--out 是目录、或所在的目录不在的拒：不写」
  MAKE_SWAP_SPEC_BREAK=max-context-unchecked  --max-context 不核是不是正整数
      「--max-context 不是正整数的拒（退 2）」
  MAKE_SWAP_SPEC_BREAK=path-escape-accepted  点名的路径绝对、归一之后落到根外也收
      「点名的路径绝对、或归一之后落到根外的拒（退 2）」
  MAKE_SWAP_SPEC_BREAK=duplicate-path-accepted  同一份文件点两次也收
      「同一份文件点两次的拒（归一之后相同也算，退 2）」
  MAKE_SWAP_SPEC_BREAK=work-root-failure-accepted  工作根取不到（多机配置没写数据根、环境里设着工作根库的弄坏开关）时当没有工作根要核，照写
      「取不到工作根的拒：多机配置没写数据根、环境里设着工作根库的弄坏开关都拒，不写」
      「命令前设 SINGLEFS_MULTI_HOST_CONFIG 指到别处的多机配置：不认，本仓自己没有多机配置的照样取不到工作根」
  MAKE_SWAP_SPEC_BREAK=work-root-from-environment  工作根照环境里的 SINGLEFS_WORK_ROOT 取（命令前设一个目录就把它认成工作根；环境里的 SINGLEFS_MULTI_HOST_CONFIG 也跟着进了工作根库）
      「工作根不认命令前的环境变量：SINGLEFS_WORK_ROOT 指到别处的 --out 拒，照多机配置的工作根写」
      「命令前设 SINGLEFS_MULTI_HOST_CONFIG 指到别处的多机配置：不认，本仓自己没有多机配置的照样取不到工作根」
  MAKE_SWAP_SPEC_BREAK=configuration-variable-honored  工作根的多机配置照环境里的 SINGLEFS_MULTI_HOST_CONFIG 取（命令前指到一份自己写的多机配置就改得了工作根）
      「命令前设 SINGLEFS_MULTI_HOST_CONFIG 指到别处的多机配置：不认，本仓自己没有多机配置的照样取不到工作根」
  MAKE_SWAP_SPEC_BREAK=out-inside-git-work-tree-accepted  --out 不核落没落在 git 工作树里（往上有没有 .git），工作树里的位置、指进工作树的符号链接都照写
      「--out 落在 git 工作树里的拒：往上有 .git（目录或文件）、经符号链接指进工作树，都拒、不写」
  MAKE_SWAP_SPEC_BREAK=temporary-file-predictable  临时文件取 <out>.<进程号>.tmp 这个可预测的名字、直接 open 写（跟随事先放好的符号链接）
      「临时文件排他新建、不跟符号链接：事先放在这个名字上的链接不被写穿」
不认识的值直接拒绝，免得拼错的开关被当成「没开」。
"""
import concurrent.futures
import difflib
import importlib.util
import json
import os
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path[:0] = [os.path.join(REPOSITORY_ROOT, ".claude", "scripts"), os.path.join(REPOSITORY_ROOT, "scripts", "hosts")]
from project_preflight import preflight  # noqa: E402
import lib_repository_copy as repository_copy  # noqa: E402

BREAK_VARIABLE = "MAKE_SWAP_SPEC_BREAK"
KNOWN_BREAK_MODES = ("reverse-check-skipped", "context-cap-ignored", "missing-base-file-accepted", "out-outside-work-root-accepted", "empty-spec-accepted",
                     "non-utf8-decoded-leniently", "out-not-file-accepted", "max-context-unchecked", "path-escape-accepted", "duplicate-path-accepted",
                     "work-root-failure-accepted", "work-root-from-environment", "configuration-variable-honored", "out-inside-git-work-tree-accepted",
                     "temporary-file-predictable")
WORK_ROOT_BREAK_VARIABLE = "SINGLEFS_WORK_ROOT_BREAK"   # .claude/hooks/lib_work_root.py 的弄坏开关变量：取工作根时设着就不取
WORK_ROOT_LIBRARY_ENTRY = ".claude/hooks/lib_work_root.py"   # 自证的假仓里要有的库：仓根条目，经 lib_repository_copy.py 的闭包与 copy_entries 拷
WORK_ROOT_VARIABLE = "SINGLEFS_WORK_ROOT"
CONFIGURATION_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"   # 多机配置的位置变量：这里不认（文件头），自证拿它造「命令前指到别处」的样本
MAX_CONTEXT_LINES = 40
EXIT_WRITTEN, EXIT_UNPLACEABLE, EXIT_USAGE, EXIT_BAD_INPUT = 0, 1, 2, 3


class Refused(Exception):
    """拒绝：code 是退出码，str 是两行，第二行以「→」开头写下一步。"""

    def __init__(self, code, text):
        super().__init__(text)
        self.code = code


def break_mode():
    return os.environ.get(BREAK_VARIABLE, "")


def read_text(root, relative, mode, base=False):
    """root 下 relative 那一份的 UTF-8 文本（按字节解，不转换换行）。弄坏开关（只给自证判红用）：missing-base-file-accepted 下基线（base）里没有的交空串，
    non-utf8-decoded-leniently 下不是 UTF-8 的按替换字符解。"""
    path = os.path.join(root, relative)
    if not os.path.isfile(path):
        if base and mode == "missing-base-file-accepted":
            return ""
        raise Refused(EXIT_BAD_INPUT, f"✗ {path} 不在或不是普通文件，规格没写\n→ 怎么办：核对 --base / --new 与相对路径；新建的文件不经规格换上，照定义第 3 步一次写全")
    with open(path, "rb") as handle:
        raw = handle.read()
    try:
        return raw.decode("utf-8", errors="replace" if mode == "non-utf8-decoded-leniently" else "strict")
    except UnicodeDecodeError as error:
        raise Refused(EXIT_BAD_INPUT, f"✗ {path} 不是 UTF-8 文本（{error}），规格没写\n→ 怎么办：hook-swap.py 只换文本文件，这一份照别的办法换") from error


def changed_blocks(base_lines, new_lines):
    """按行比出的不相同的块 → [(基线起, 基线止, 新版起, 新版止)]（下标从 0 起，止不含）。"""
    matcher = difflib.SequenceMatcher(None, base_lines, new_lines, autojunk=False)
    return [(i1, i2, j1, j2) for tag, i1, i2, j1, j2 in matcher.get_opcodes() if tag != "equal"]


def build_items(relative, base_lines, new_lines, blocks, contexts, mode, checked=True):
    """照每一项此刻的上下文行数依次造项 → (项表, None) 或 (None, (第几项, 不满足的那一条))。每一项按前面几项施加之后的文本取。
    checked=False 只造不核（只给弄坏开关 context-cap-ignored 用）；弄坏开关 reverse-check-skipped 下不核新串。"""
    current, offset, items, after_each = list(base_lines), 0, [], []
    for index, (i1, i2, j1, j2) in enumerate(blocks):
        start, end, context = i1 + offset, i2 + offset, contexts[index]
        before, after = current[max(0, start - context):start], current[end:end + context]
        old, new = "".join(before + current[start:end] + after), "".join(before + new_lines[j1:j2] + after)
        if checked and (not old or not new):
            return None, (index, "旧串或新串是空的")
        hits = "".join(current).count(old)
        if checked and hits != 1:
            return None, (index, f"旧串在施加到这一步时的文本里命中 {hits} 次")
        current[start:end] = new_lines[j1:j2]
        offset += (j2 - j1) - (i2 - i1)
        items.append({"file": relative, "old": old, "new": new})
        after_each.append("".join(current))
    if not checked or mode == "reverse-check-skipped":
        return items, None
    final = "".join(current)
    for index, item in enumerate(items):
        hits = after_each[index].count(item["new"])
        if hits != 1:
            return None, (index, f"新串在这一步施加之后的文本里命中 {hits} 次（倒着施加到这一步时定不了位）")
        hits = final.count(item["new"])
        if hits != 1:
            return None, (index, f"新串在整份规格施加完之后的文本里命中 {hits} 次")
    return items, None


def file_items(relative, base_text, new_text, max_context, mode):
    """一份文件的规格项：不满足的那一项多带一行上下文再造，带到 max_context 还不满足就拒（Refused，退 1）。"""
    base_lines, new_lines = base_text.splitlines(keepends=True), new_text.splitlines(keepends=True)
    blocks = changed_blocks(base_lines, new_lines)
    contexts = [0] * len(blocks)
    while True:
        items, problem = build_items(relative, base_lines, new_lines, blocks, contexts, mode)
        if problem is None:
            return items
        index, reason = problem
        if contexts[index] >= max_context:
            if mode == "context-cap-ignored":   # 弄坏开关：带满上限照交，只给自证判红用
                return build_items(relative, base_lines, new_lines, blocks, contexts, mode, checked=False)[0]
            i1, i2, j1, j2 = blocks[index]
            raise Refused(EXIT_UNPLACEABLE, f"✗ {relative} 基线第 {i1 + 1} 行起那一处（基线 {i2 - i1} 行 → 新版 {j2 - j1} 行）前后各带到 {max_context} 行还定不了位：{reason}，规格没写\n"
                                            "→ 怎么办：这一处附近有成片相同的行；把新版里这一处改得与别处不同、挪开，或加 --max-context 放宽上限再跑")
        contexts[index] += 1


def apply_in_order(texts, items, reverse=False):
    """依次施加（reverse 时新旧对调、次序倒排），每处要恰好命中 1 次，同 hook-swap.py 的 apply_edits → (新文本表, None) 或 (None, 问题一句)。"""
    texts = dict(texts)
    numbered = list(enumerate(items, 1))
    for number, item in (reversed(numbered) if reverse else numbered):
        old, new = (item["new"], item["old"]) if reverse else (item["old"], item["new"])
        hits = texts[item["file"]].count(old)
        if hits != 1:
            return None, f"第 {number} 处 {item['file']}：{'倒着' if reverse else '正着'}施加时期望命中 1 次，实际 {hits}"
        texts[item["file"]] = texts[item["file"]].replace(old, new)
    return texts, None


def enclosing_git_work_tree(path):
    """path（已解开符号链接的绝对路径）往上第一层有 .git（目录或文件）的目录；一层都没有交 None。"""
    directory = path
    while True:
        if os.path.lexists(os.path.join(directory, ".git")):
            return directory
        parent = os.path.dirname(directory)
        if parent == directory:
            return None
        directory = parent


def checked_out_path(out, work_roots, mode):
    """--out 的绝对路径；不在工作根下（根本身不收）、是目录、所在目录不在、落在 git 工作树里的拒（Refused，退 3）。work_roots 是 None（工作根取不到，开关 work-root-failure-accepted 下）时不核工作根。
    落在 git 工作树里：--out 所在目录的真实路径与 --out 自己的真实路径，往上任何一层有 .git；这样主工作区、git worktree、仓副本里的位置与指进它们的符号链接都写不进，
    不用认出哪一棵树是副本。弄坏开关：out-outside-work-root-accepted 下不核工作根，out-not-file-accepted 下不核 --out 是不是目录、所在目录在不在，
    out-inside-git-work-tree-accepted 下不核落没落在 git 工作树里。"""
    absolute = os.path.abspath(out)
    parent = os.path.realpath(os.path.dirname(absolute))
    if mode != "out-not-file-accepted" and (os.path.isdir(absolute) or not os.path.isdir(parent)):
        raise Refused(EXIT_BAD_INPUT, f"✗ --out {out} 是目录，或它所在的目录 {parent} 不在，规格没写\n→ 怎么办：--out 写草稿目录里的一份 .json 文件路径（目录先建好）")
    if mode != "out-inside-git-work-tree-accepted":
        for real in dict.fromkeys((parent, os.path.realpath(absolute))):
            work_tree = enclosing_git_work_tree(real)
            if work_tree is not None:
                raise Refused(EXIT_BAD_INPUT, f"✗ --out {out} 的真实路径 {real} 落在 git 工作树里（{work_tree} 有 .git），规格没写\n"
                                              "→ 怎么办：--out 写进派发提示给的草稿目录里不在任何 git 工作树内的位置（仓副本、git worktree 里放不了；指进它们的符号链接也不行）")
    if work_roots is not None and mode != "out-outside-work-root-accepted":
        roots = [os.path.realpath(root) for root in work_roots]
        if not any(parent.startswith(root + os.sep) for root in roots):
            raise Refused(EXIT_BAD_INPUT, f"✗ --out {out} 不在工作根下（认的根：{'、'.join(roots)}），规格没写\n"
                                          "→ 怎么办：写进派发提示给的草稿目录（工作根 ~/.local/share/singlefs/work/<轮>/ 下；工作根照多机配置取，命令前设 SINGLEFS_WORK_ROOT 改不了它）")
    return absolute


def write_json_atomically(out_path, spec, mode):
    """spec 写成 out_path：同目录里 tempfile.mkstemp 排他新建临时文件（O_EXCL，名字带随机段，不跟随符号链接），写完刷成 0644 再改名；
    out_path 自己是符号链接时改名换掉链接本身、不写穿。出错时删掉临时文件再抛。弄坏开关 temporary-file-predictable：临时文件取 <out>.<进程号>.tmp、直接 open（跟随符号链接）。"""
    if mode == "temporary-file-predictable":
        temporary = f"{out_path}.{os.getpid()}.tmp"
        handle = open(temporary, "w", encoding="utf-8")
    else:
        descriptor, temporary = tempfile.mkstemp(prefix=os.path.basename(out_path) + ".", suffix=".tmp", dir=os.path.dirname(out_path))
        handle = os.fdopen(descriptor, "w", encoding="utf-8")
    try:
        with handle:
            os.fchmod(handle.fileno(), 0o644)
            json.dump(spec, handle, ensure_ascii=False, indent=1)
            handle.write("\n")
        os.replace(temporary, out_path)
    except BaseException:
        if os.path.lexists(temporary):
            os.unlink(temporary)
        raise


def generate(base_root, new_root, out, relatives, max_context, work_roots, mode=""):
    """算规格、整份核、写 --out → (退出码, 提示行)。拒绝都在写之前，拒了一个字节不写。"""
    try:
        out_path = checked_out_path(out, work_roots, mode)
        base_texts, new_texts = {}, {}
        for relative in relatives:
            base_texts[relative] = read_text(base_root, relative, mode, base=True)
            new_texts[relative] = read_text(new_root, relative, mode)
        spec, unchanged = [], []
        for relative in relatives:
            if base_texts[relative] == new_texts[relative]:
                unchanged.append(relative)
                continue
            spec += file_items(relative, base_texts[relative], new_texts[relative], max_context, mode)
        if not spec and mode != "empty-spec-accepted":
            raise Refused(EXIT_BAD_INPUT, f"✗ 点名的 {len(relatives)} 份文件基线与新版逐字相同，一处改动都没有，规格没写\n→ 怎么办：核对 --base 与 --new 是不是写反了、写成了同一棵树")
        if mode != "reverse-check-skipped":
            forward, problem = apply_in_order(base_texts, spec)
            if problem is None and forward != new_texts:
                problem = "正着依次施加之后与新版不同：" + "、".join(relative for relative in relatives if forward[relative] != new_texts[relative])
            if problem is None:
                backward, problem = apply_in_order(forward, spec, reverse=True)
                if problem is None and backward != base_texts:
                    problem = "倒着依次施加之后回不到基线：" + "、".join(relative for relative in relatives if backward[relative] != base_texts[relative])
            if problem is not None:
                raise Refused(EXIT_UNPLACEABLE, f"✗ 整份核不过（{problem}），规格没写\n→ 怎么办：这是生成的毛病，不是规格写法的毛病；把基线、新版与这条命令交主 agent")
    except Refused as refusal:
        return refusal.code, str(refusal).splitlines()
    write_json_atomically(out_path, spec, mode)
    changed = len(relatives) - len(unchanged)
    lines = [f"✓ 写出 {out_path}：{changed} 份文件 {len(spec)} 处，正着依次施加等于新版、倒着依次施加回到基线，每处恰好命中 1 次"]
    if unchanged:
        lines.append(f"  逐字相同、没出项的 {len(unchanged)} 份：{'、'.join(unchanged)}")
    return EXIT_WRITTEN, lines


def work_roots_now(repository_root, environment, mode):
    """此刻认的工作根 → 一项的元组：repository_root 里 .claude/hooks/lib_work_root.py 的 shared_data_path("work")，在空环境里取（SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT、
    SINGLEFS_MULTI_HOST_CONFIG 都不看，只读那一份库所在仓的多机配置），所以命令前设什么变量都改不了 --out 能写到哪。
    environment 里设着工作根库的弄坏开关、库读不了、多机配置没写数据根都拒（Refused，退 3）。不认 repository_root 是不是仓副本（--out 落在 git 工作树里的拒在 checked_out_path）。
    弄坏开关：work-root-failure-accepted 下取不到交 None（--out 不核工作根），work-root-from-environment 下照 environment 的 SINGLEFS_WORK_ROOT 取，
    configuration-variable-honored 下把 environment 的 SINGLEFS_MULTI_HOST_CONFIG 交给工作根库。"""
    try:
        if environment.get(WORK_ROOT_BREAK_VARIABLE):
            raise ValueError(f"环境里设着 {WORK_ROOT_BREAK_VARIABLE}（工作根库的弄坏开关，只在自证里认）")
        path = os.path.join(repository_root, ".claude", "hooks", "lib_work_root.py")
        if not os.path.isfile(path):
            raise ValueError(f"{path} 不在")
        specification = importlib.util.spec_from_file_location("lib_work_root", path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        if mode == "work-root-from-environment":
            return (module.configured_work_root(environment),)
        if mode == "configuration-variable-honored":
            return (module.shared_data_path("work", {CONFIGURATION_VARIABLE: environment.get(CONFIGURATION_VARIABLE, "")}),)
        return (module.shared_data_path("work", {}),)
    except (ValueError, OSError, SyntaxError) as error:   # DataRootUnconfigured 是 ValueError 的子类
        if mode == "work-root-failure-accepted":
            return None
        raise Refused(EXIT_BAD_INPUT, f"✗ 取不到工作根（{error}），规格没写\n"
                                      "→ 怎么办：主工作区里在多机配置（主工作树根的 multi-host.env）写 SHARED_DATA_ROOT，写法见 .claude/hooks/lib_work_root.py 文件头「数据根」；"
                                      "在仓副本里起的（副本不拷多机配置）改起主工作区那一份：python3 <主工作区>/scripts/gate/make-swap-spec.py --base <基线根> --new <新版根> "
                                      f"--out <规格.json> <相对路径>…；{WORK_ROOT_BREAK_VARIABLE} 设着的先 unset；命令前设 {CONFIGURATION_VARIABLE} 不算数（取的是这份工具所在仓自己的多机配置）") from error


# ---- 自证：每格在自己的临时目录里造基线与新版两棵小树，直接调 generate（工作根给那一格的临时目录），互不依赖，按每台默认重活线程数并行 ----
def write_tree(root, files):
    """files：相对路径 → 内容（str 按 UTF-8 写，bytes 原样写）。"""
    for relative, content in files.items():
        os.makedirs(os.path.dirname(os.path.join(root, relative)), exist_ok=True)
        if isinstance(content, bytes):
            with open(os.path.join(root, relative), "wb") as handle:
                handle.write(content)
        else:
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(content)


def selftest_case(base_files, new_files, relatives, max_context=MAX_CONTEXT_LINES, out_kind="file"):
    """在临时目录里造两棵树、跑一次 generate（工作根是这一格的 work/，草稿目录是它下面的 scratch/）→ (退出码, 提示行, 写出的规格或 None)。
    out_kind：file 是 scratch/ 里的 spec.json，outside 在工作根之外，directory 是个已建好的目录，missing-parent 的父目录不在。
    generate 抛了错（弄坏开关让写这一步才出错）交退出码 None、提示行写抛了什么。"""
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as work:
        base_root, new_root, work_root = (os.path.join(work, name) for name in ("base", "new", "work"))
        for root, files in ((base_root, base_files), (new_root, new_files)):
            write_tree(root, files)
        scratch = os.path.join(work_root, "scratch")
        os.makedirs(scratch)
        out = {"file": os.path.join(scratch, "spec.json"), "outside": os.path.join(work, "spec.json"), "directory": os.path.join(scratch, "spec.json"),
               "missing-parent": os.path.join(scratch, "absent", "spec.json")}[out_kind]
        if out_kind == "directory":
            os.makedirs(out)
        try:
            code, lines = generate(base_root, new_root, out, relatives, max_context, (work_root,), break_mode())
        except Exception as error:   # noqa: BLE001  自证要把任何一种出错都记成这一格的问题，不让一格的异常带倒整份自证
            code, lines = None, [f"✗ generate 抛了 {type(error).__name__}：{error}"]
        spec = None
        if os.path.isfile(out):
            with open(out, encoding="utf-8") as handle:
                spec = json.load(handle)
        return code, lines, spec


def replay_problems(spec, base_files, new_files):
    """格里自己再核一遍写出的规格：正着施加等于新版、倒着施加回到基线 → 问题列表。"""
    if not spec:
        return ["没写出规格或规格是空的"]
    relatives = sorted({item["file"] for item in spec})
    forward, problem = apply_in_order({relative: base_files[relative] for relative in relatives}, spec)
    if problem:
        return [problem]
    if forward != {relative: new_files[relative] for relative in relatives}:
        return ["正着施加之后与新版不同"]
    backward, problem = apply_in_order(forward, spec, reverse=True)
    if problem:
        return [problem]
    return [] if backward == {relative: base_files[relative] for relative in relatives} else ["倒着施加之后回不到基线"]


def cell_ordinary_changes():
    base = {"scripts/tool.py": "def alpha():\n    return 1\n\n\ndef beta():\n    return 2\n", "scripts/tool.sh": "keep\ndrop\nkeep too\n"}
    new = {"scripts/tool.py": "def alpha():\n    return 10\n\n\ndef beta():\n    return 2\n\n\ndef gamma():\n    return 3\n", "scripts/tool.sh": "keep\nkeep too\n"}
    code, lines, spec = selftest_case(base, new, ["scripts/tool.py", "scripts/tool.sh"])
    problems = [] if code == EXIT_WRITTEN else [f"退 {code}（应当 0）"]
    problems += replay_problems(spec, base, new)
    if spec is not None and len(spec) != 3:
        problems.append(f"出了 {len(spec)} 处（应当 3：改一行、末尾加一段、删一行）")
    if not any("2 份文件 3 处" in line for line in lines):
        problems.append("成功句里没有「2 份文件 3 处」")
    return problems, lines


def cell_new_string_twice():
    # 真事的样子：只带改动行本身时，新串在施加之后的文件里另有一处一模一样的行，倒着施加命中 2 次
    base = {"task-server/selftest.py": "alpha\nold\nbeta\nnew\ngamma\n"}
    new = {"task-server/selftest.py": "alpha\nnew\nbeta\nnew\ngamma\n"}
    code, lines, spec = selftest_case(base, new, ["task-server/selftest.py"])
    problems = [] if code == EXIT_WRITTEN else [f"退 {code}（应当 0）"]
    problems += replay_problems(spec, base, new)
    if spec and spec[0]["old"] == "old\n":
        problems.append("那一处没多带上下文（旧串只是改动行本身）")
    return problems, lines


def cell_unplaceable():
    # 两行一循环的 30 行里把第 16 行（一个 b）换成 c：旧串带多少行上下文都不唯一。改动行要是个新出现的行：
    # 全是同一行的文件里改一行，difflib 会切成「开头插 15 行、中间删 15 行」两块，点名的位置就不是改的那一行了
    cycle = ["a\n", "b\n"] * 15
    changed = list(cycle)
    changed[15] = "c\n"
    code, lines, spec = selftest_case({"scripts/flat.txt": "".join(cycle)}, {"scripts/flat.txt": "".join(changed)}, ["scripts/flat.txt"], max_context=3)
    problems = [] if code == EXIT_UNPLACEABLE else [f"退 {code}（应当 1）"]
    if spec is not None:
        problems.append("拒了还是写了规格")
    if not any("scripts/flat.txt 基线第 16 行起那一处" in line and "前后各带到 3 行" in line for line in lines):
        problems.append("没点名「scripts/flat.txt 基线第 16 行起那一处」与上限 3 行")
    return problems, lines


def cell_missing_base_file():
    code, lines, spec = selftest_case({"scripts/other.py": "x = 1\n"}, {"scripts/missing.py": "x = 2\n"}, ["scripts/missing.py"])
    problems = [] if code == EXIT_BAD_INPUT else [f"退 {code}（应当 3）"]
    if spec is not None:
        problems.append("拒了还是写了规格")
    if not any("missing.py 不在或不是普通文件" in line for line in lines):
        problems.append("没点名基线里没有的 scripts/missing.py")
    return problems, lines


def cell_out_outside_work_root():
    code, lines, spec = selftest_case({"scripts/tool.py": "x = 1\n"}, {"scripts/tool.py": "x = 2\n"}, ["scripts/tool.py"], out_kind="outside")
    problems = [] if code == EXIT_BAD_INPUT else [f"退 {code}（应当 3）"]
    if spec is not None:
        problems.append("拒了还是写了规格")
    return problems, lines


def cell_nothing_changed():
    code, lines, spec = selftest_case({"scripts/tool.py": "x = 1\n"}, {"scripts/tool.py": "x = 1\n"}, ["scripts/tool.py"])
    problems = [] if code == EXIT_BAD_INPUT else [f"退 {code}（应当 3）"]
    if spec is not None:
        problems.append("拒了还是写了规格")
    return problems, lines


def cell_new_file_not_utf8():
    # 新版里那一份有一个孤零零的 0xff 字节：不是 UTF-8
    code, lines, spec = selftest_case({"scripts/tool.py": "x = 1\n"}, {"scripts/tool.py": b"x = \xff\n"}, ["scripts/tool.py"])
    problems = [] if code == EXIT_BAD_INPUT else [f"退 {code}（应当 3）"]
    if spec is not None:
        problems.append("拒了还是写了规格")
    if not any("scripts/tool.py 不是 UTF-8 文本" in line for line in lines):
        problems.append("没点名「scripts/tool.py 不是 UTF-8 文本」")
    return problems, lines


def cell_out_is_directory_or_parent_missing():
    problems, shown = [], []
    for out_kind, name in (("directory", "--out 是目录"), ("missing-parent", "--out 所在的目录不在")):
        code, lines, spec = selftest_case({"scripts/tool.py": "x = 1\n"}, {"scripts/tool.py": "x = 2\n"}, ["scripts/tool.py"], out_kind=out_kind)
        shown = shown or lines
        if code != EXIT_BAD_INPUT:
            problems.append(f"{name}：退 {code}（应当 3），输出头一行 {lines[:1]}")
        if spec is not None:
            problems.append(f"{name}：拒了还是写了规格")
        if code == EXIT_BAD_INPUT and not any("是目录，或它所在的目录" in line for line in lines):
            problems.append(f"{name}：拒绝说明里没有「是目录，或它所在的目录」")
    return problems, shown


def parse_outcome(arguments):
    """parse_arguments 对一串参数的结局 → (退出码或 None（没拒）, 拒绝说明, 相对路径表或 None)。"""
    try:
        return None, "", parse_arguments(["--base", "base", "--new", "new", "--out", "out.json", *arguments], break_mode())[1]
    except Refused as refusal:
        return refusal.code, str(refusal), None


def cell_max_context_not_positive_integer():
    problems = []
    for value in ("0", "abc", "-1", "1.5", "", "²"):   # 最后一个上标二：isdigit() 为真、int() 抛错
        code, text, _ = parse_outcome(["--max-context", value, "scripts/tool.py"])
        if code != EXIT_USAGE or "--max-context" not in text:
            problems.append(f"--max-context {value!r}：退 {code}（应当 2）并点名 --max-context")
    code, _, relatives = parse_outcome(["--max-context", "3", "scripts/tool.py"])
    if code is not None or relatives != ["scripts/tool.py"]:
        problems.append(f"--max-context 3 被拒了（退 {code}）")
    return problems, []


def cell_path_absolute_or_outside_root():
    problems = []
    for written in ("/etc/passwd", "../tool.py", "scripts/../../tool.py", ".", "scripts/.."):
        code, text, _ = parse_outcome([written])
        if code != EXIT_USAGE or written not in text:
            problems.append(f"路径 {written!r}：退 {code}（应当 2）并点名它")
    code, _, relatives = parse_outcome(["scripts/./tool.py"])
    if code is not None or relatives != ["scripts/tool.py"]:
        problems.append(f"路径 'scripts/./tool.py'：应当归一成 scripts/tool.py 收下，实际退 {code}、得 {relatives}")
    return problems, []


def cell_same_file_twice():
    problems = []
    for pair in (["scripts/tool.py", "scripts/tool.py"], ["scripts/tool.py", "scripts/./tool.py"]):
        code, text, _ = parse_outcome(pair)
        if code != EXIT_USAGE or "点了两次" not in text:
            problems.append(f"{pair}：退 {code}（应当 2）并说「点了两次」")
    code, _, relatives = parse_outcome(["scripts/tool.py", "scripts/other.py"])
    if code is not None or relatives != ["scripts/tool.py", "scripts/other.py"]:
        problems.append(f"两份不同的文件被拒了（退 {code}）")
    return problems, []


def sandbox_repository(work, configuration_lines):
    """work 下造一棵小仓 → 仓根：.claude/hooks/lib_work_root.py 是真库的一份拷贝（经 lib_repository_copy.py 的 copy_closure 算、copy_entries 拷），多机配置 multi-host.env 照 configuration_lines 写（None 不写这份文件），
    放一个空的 .git 目录（scripts/hosts/minimal-copy.py 建的副本在副本根 git init，也带它）。"""
    root = os.path.join(work, "repository")
    os.makedirs(os.path.join(root, ".git"))
    closure = repository_copy.copy_closure(REPOSITORY_ROOT, [(WORK_ROOT_LIBRARY_ENTRY, "自证的假仓里取工作根用的库")], workspace=WORK_ROOT_LIBRARY_ENTRY)
    repository_copy.copy_entries(REPOSITORY_ROOT, root, closure.entries)
    if configuration_lines is not None:
        with open(os.path.join(root, "multi-host.env"), "w", encoding="utf-8") as handle:
            handle.write("\n".join(configuration_lines) + "\n")
    return root


def sandbox_trees(work):
    """work 下的 base/、new/ 两棵各有一份改了一行的 scripts/tool.py 的小树 → (基线根, 新版根)。"""
    base_root, new_root = os.path.join(work, "base"), os.path.join(work, "new")
    write_tree(base_root, {"scripts/tool.py": "x = 1\n"})
    write_tree(new_root, {"scripts/tool.py": "x = 2\n"})
    return base_root, new_root


def configured_data_root_lines(work):
    """多机配置的两行：共享根是 work/shared，数据根是它下面的 data（工作根就是 work/shared/data/work）。"""
    return [f"NFS_SHARED_ROOTS={os.path.join(work, 'shared')}", f"SHARED_DATA_ROOT={os.path.join(work, 'shared', 'data')}"]


def cell_work_root_unobtainable():
    problems, shown = [], []
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as scratch_base:
        work = os.path.realpath(scratch_base)
        base_root, new_root = sandbox_trees(work)
        out_directory = os.path.join(work, "scratch")   # 工作根之外：只有开关把工作根的核对放掉，写才会落在这里
        os.makedirs(out_directory)
        samples = (("多机配置里没写 SHARED_DATA_ROOT", configured_data_root_lines(work)[:1], {}),
                   ("没有多机配置", None, {}),
                   ("环境里设着工作根库的弄坏开关", configured_data_root_lines(work), {WORK_ROOT_BREAK_VARIABLE: "new-root-unknown"}))
        for number, (name, configuration, environment) in enumerate(samples):
            sample_directory = os.path.join(work, f"sample-{number}")
            os.makedirs(sample_directory)
            root = sandbox_repository(sample_directory, configuration)
            out = os.path.join(out_directory, f"spec-{number}.json")
            code, lines = run(["--base", base_root, "--new", new_root, "--out", out, "scripts/tool.py"], root, environment)
            shown = shown or lines
            if code != EXIT_BAD_INPUT or not any("取不到工作根" in line for line in lines):
                problems.append(f"{name}：退 {code}（应当 3）并说「取不到工作根」，输出头一行 {lines[:1]}")
            if os.path.exists(out):
                problems.append(f"{name}：拒了还是写了规格")
    return problems, shown


def cell_work_root_ignores_environment():
    problems, shown = [], []
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as scratch_base:
        work = os.path.realpath(scratch_base)
        base_root, new_root = sandbox_trees(work)
        root = sandbox_repository(work, configured_data_root_lines(work))
        configured_scratch, fake_scratch = os.path.join(work, "shared", "data", "work", "scratch"), os.path.join(work, "fakework", "scratch")
        os.makedirs(configured_scratch)
        os.makedirs(fake_scratch)
        environment = {WORK_ROOT_VARIABLE: os.path.join(work, "fakework")}
        fake_out, configured_out = os.path.join(fake_scratch, "spec.json"), os.path.join(configured_scratch, "spec.json")
        code, lines = run(["--base", base_root, "--new", new_root, "--out", fake_out, "scripts/tool.py"], root, environment)
        shown = lines
        if code != EXIT_BAD_INPUT or not any("不在工作根下" in line for line in lines):
            problems.append(f"命令前设 SINGLEFS_WORK_ROOT 指到别处：--out 写在那里，退 {code}（应当 3）并说「不在工作根下」，输出头一行 {lines[:1]}")
        if os.path.exists(fake_out):
            problems.append("命令前设 SINGLEFS_WORK_ROOT 指到别处：拒了还是写了规格")
        code, lines = run(["--base", base_root, "--new", new_root, "--out", configured_out, "scripts/tool.py"], root, environment)
        if code != EXIT_WRITTEN or not os.path.isfile(configured_out):
            problems.append(f"同样设着 SINGLEFS_WORK_ROOT，--out 写在多机配置的工作根下：退 {code}（应当 0）并写出规格，输出头一行 {lines[:1]}")
    return problems, shown


def cell_configuration_variable_not_honored():
    """命令前设 SINGLEFS_MULTI_HOST_CONFIG 指到一份写好的多机配置：本仓自己没有多机配置的照样取不到工作根、不写规格（照那份配置，--out 落在它的工作根下，写得出来）。"""
    problems, shown = [], []
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as scratch_base:
        work = os.path.realpath(scratch_base)
        base_root, new_root = sandbox_trees(work)
        written = sandbox_repository(os.path.join(work, "written"), configured_data_root_lines(work))
        root = sandbox_repository(os.path.join(work, "own"), None)
        out_directory = os.path.join(work, "shared", "data", "work", "scratch")
        os.makedirs(out_directory)
        out = os.path.join(out_directory, "spec.json")
        environment = {CONFIGURATION_VARIABLE: os.path.join(written, "multi-host.env")}
        code, lines = run(["--base", base_root, "--new", new_root, "--out", out, "scripts/tool.py"], root, environment)
        shown = lines
        if code != EXIT_BAD_INPUT or not any("取不到工作根" in line for line in lines):
            problems.append(f"命令前设 {CONFIGURATION_VARIABLE} 指到一份写好的多机配置：退 {code}（应当 3）并说「取不到工作根」，输出头一行 {lines[:1]}")
        if os.path.exists(out):
            problems.append(f"命令前设 {CONFIGURATION_VARIABLE} 指到一份写好的多机配置：拒了还是写了规格")
    return problems, shown


def cell_out_inside_git_work_tree_refused():
    """--out 落在 git 工作树里（往上有 .git：目录、文件）、经符号链接指进工作树（所在目录是链接、--out 自己是链接）的拒，不写；工作树之外的 --out 照写。"""
    problems, shown = [], []
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as scratch_base:
        work = os.path.realpath(scratch_base)
        base_root, new_root = sandbox_trees(work)
        work_root = os.path.join(work, "work")
        scratch = os.path.join(work_root, "scratch")
        os.makedirs(scratch)
        tree_with_directory = os.path.join(work_root, "tree-directory")   # .git 是目录（主工作区、副本）
        tree_with_file = os.path.join(work_root, "tree-file")             # .git 是文件（git worktree）
        for tree, marker in ((tree_with_directory, "directory"), (tree_with_file, "file")):
            os.makedirs(os.path.join(tree, "sub"))
            if marker == "directory":
                os.makedirs(os.path.join(tree, ".git"))
            else:
                write_tree(tree, {".git": "gitdir: /nonexistent\n"})
        os.symlink(os.path.join(tree_with_directory, "sub"), os.path.join(scratch, "link-to-tree"))
        os.symlink(os.path.join(tree_with_directory, "sub", "inside.json"), os.path.join(scratch, "spec-link.json"))
        samples = (("所在目录往上有 .git 目录", os.path.join(tree_with_directory, "sub", "spec.json"), os.path.join(tree_with_directory, "sub", "spec.json")),
                   ("所在目录往上有 .git 文件（git worktree）", os.path.join(tree_with_file, "sub", "spec.json"), os.path.join(tree_with_file, "sub", "spec.json")),
                   ("所在目录是指进工作树的符号链接", os.path.join(scratch, "link-to-tree", "spec.json"), os.path.join(tree_with_directory, "sub", "spec.json")),
                   ("--out 自己是指进工作树的符号链接", os.path.join(scratch, "spec-link.json"), os.path.join(tree_with_directory, "sub", "inside.json")))
        for name, out, landing in samples:
            code, lines = generate(base_root, new_root, out, ["scripts/tool.py"], MAX_CONTEXT_LINES, (work_root,), break_mode())
            shown = shown or lines
            if code != EXIT_BAD_INPUT or not any("落在 git 工作树里" in line for line in lines):
                problems.append(f"{name}：退 {code}（应当 3）并说「落在 git 工作树里」，输出头一行 {lines[:1]}")
            if os.path.exists(landing):
                problems.append(f"{name}：拒了还是写了规格")
        out = os.path.join(scratch, "spec.json")
        code, lines = generate(base_root, new_root, out, ["scripts/tool.py"], MAX_CONTEXT_LINES, (work_root,), break_mode())
        if code != EXIT_WRITTEN or not os.path.isfile(out):
            problems.append(f"工作树之外的草稿目录：退 {code}（应当 0）并写出规格，输出头一行 {lines[:1]}")
    return problems, shown


def cell_copy_with_handwritten_configuration_not_recognized():
    """仓副本（根上带 .git）里起、副本根上有一份手写的多机配置：工具不认副本，工作根照那份配置取，--out 落在那份配置的工作根下（不在 git 工作树里）照常出规格，
    落在那份配置的工作根之外的拒。"""
    problems, shown = [], []
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as scratch_base:
        work = os.path.realpath(scratch_base)
        base_root, new_root = sandbox_trees(work)
        copy_root = sandbox_repository(os.path.join(work, "copy"), configured_data_root_lines(work))
        configured_scratch = os.path.join(work, "shared", "data", "work", "scratch")
        os.makedirs(configured_scratch)
        elsewhere = os.path.join(work, "elsewhere")
        os.makedirs(elsewhere)
        inside_out, outside_out = os.path.join(configured_scratch, "spec.json"), os.path.join(elsewhere, "spec.json")
        code, lines = run(["--base", base_root, "--new", new_root, "--out", inside_out, "scripts/tool.py"], copy_root, {})
        shown = lines
        if code != EXIT_WRITTEN or not os.path.isfile(inside_out):
            problems.append(f"副本根上手写的多机配置定出的工作根下：退 {code}（应当 0）并写出规格，输出头一行 {lines[:1]}")
        code, lines = run(["--base", base_root, "--new", new_root, "--out", outside_out, "scripts/tool.py"], copy_root, {})
        if code != EXIT_BAD_INPUT or not any("不在工作根下" in line for line in lines):
            problems.append(f"那份配置的工作根之外：退 {code}（应当 3）并说「不在工作根下」，输出头一行 {lines[:1]}")
        if os.path.exists(outside_out):
            problems.append("那份配置的工作根之外：拒了还是写了规格")
    return problems, shown


def cell_temporary_file_not_followed():
    problems = []
    with tempfile.TemporaryDirectory(prefix="make-swap-spec-selftest-") as work:
        base_root, new_root = sandbox_trees(work)
        work_root = os.path.join(work, "work")
        scratch = os.path.join(work_root, "scratch")
        os.makedirs(scratch)
        out = os.path.join(scratch, "spec.json")
        victims = {"老名字的临时文件": os.path.join(work, "victim-temporary.txt"), "--out 本身": os.path.join(work, "victim-out.txt")}
        for path in victims.values():
            write_tree(os.path.dirname(path), {os.path.basename(path): "victim\n"})
        os.symlink(victims["老名字的临时文件"], f"{out}.{os.getpid()}.tmp")   # 事先放好的链接：放在可预测的临时文件名上
        os.symlink(victims["--out 本身"], out)                                # 与 --out 自己是个指向别处的链接
        code, lines = generate(base_root, new_root, out, ["scripts/tool.py"], MAX_CONTEXT_LINES, (work_root,), break_mode())
        if code != EXIT_WRITTEN:
            problems.append(f"退 {code}（应当 0），输出头一行 {lines[:1]}")
        for name, path in victims.items():
            with open(path, encoding="utf-8") as handle:
                if handle.read() != "victim\n":
                    problems.append(f"事先放好的链接被写穿了：{name}的链接目标内容变了")
        if os.path.islink(out) or not os.path.isfile(out):
            problems.append("--out 写出之后不是普通文件（还是一条链接）")
        leftovers = sorted(set(os.listdir(scratch)) - {"spec.json", f"spec.json.{os.getpid()}.tmp"})
        if leftovers:
            problems.append(f"草稿目录里留下临时文件 {leftovers}")
    return problems, lines


SELFTEST_CELLS = (
    ("普通改动出规格（改一行、末尾加一段、删一行，两份文件）：正着倒着都核过", cell_ordinary_changes),
    ("新串在施加之后出现两次的那一形：自动多带上下文，正着倒着都核过", cell_new_string_twice),
    ("带到上限仍定不了位的拒：点名那一处，不写规格", cell_unplaceable),
    ("基线里没有点名的文件的拒：点名它，不写规格", cell_missing_base_file),
    ("--out 不在工作根下的拒：不写", cell_out_outside_work_root),
    ("基线与新版逐字相同、一处改动都没有的拒：不写", cell_nothing_changed),
    ("新版不是 UTF-8 文本的拒：点名它，不写规格", cell_new_file_not_utf8),
    ("--out 是目录、或所在的目录不在的拒：不写", cell_out_is_directory_or_parent_missing),
    ("--max-context 不是正整数的拒（退 2）", cell_max_context_not_positive_integer),
    ("点名的路径绝对、或归一之后落到根外的拒（退 2）", cell_path_absolute_or_outside_root),
    ("同一份文件点两次的拒（归一之后相同也算，退 2）", cell_same_file_twice),
    ("取不到工作根的拒：多机配置没写数据根、环境里设着工作根库的弄坏开关都拒，不写", cell_work_root_unobtainable),
    ("工作根不认命令前的环境变量：SINGLEFS_WORK_ROOT 指到别处的 --out 拒，照多机配置的工作根写", cell_work_root_ignores_environment),
    ("命令前设 SINGLEFS_MULTI_HOST_CONFIG 指到别处的多机配置：不认，本仓自己没有多机配置的照样取不到工作根", cell_configuration_variable_not_honored),
    ("--out 落在 git 工作树里的拒：往上有 .git（目录或文件）、经符号链接指进工作树，都拒、不写", cell_out_inside_git_work_tree_refused),
    ("仓副本里起不认副本：副本根上手写的多机配置定出的工作根下照常出规格，工作根之外的拒", cell_copy_with_handwritten_configuration_not_recognized),
    ("临时文件排他新建、不跟符号链接：事先放在这个名字上的链接不被写穿", cell_temporary_file_not_followed),
)


def selftest_threads():
    """每台默认重活线程数（scripts/hosts/heavy_threads.py）；取不到往标准错误打一行、一路跑。"""
    from heavy_threads import HeavyThreadDefaultError, default_heavy_threads
    try:
        return max(1, int(default_heavy_threads()))
    except HeavyThreadDefaultError as error:
        print(f"  ! 取不到每台默认重活线程数（{error}），自证一路跑", file=sys.stderr)
        return 1


def selftest():
    mode = break_mode()
    if mode and mode not in KNOWN_BREAK_MODES:
        print(f"  ✗ {BREAK_VARIABLE}={mode} 不认识，只认 {'、'.join(KNOWN_BREAK_MODES)}")
        print(f"  → 怎么办：照文件头的开关表改 {BREAK_VARIABLE} 再跑")
        return EXIT_USAGE
    if mode:
        print(f"  ! {BREAK_VARIABLE}={mode} 生效：故意走坏的写法，只给自证判红用", file=sys.stderr)
    threads = min(selftest_threads(), len(SELFTEST_CELLS))
    with concurrent.futures.ThreadPoolExecutor(max_workers=threads) as pool:
        results = list(pool.map(lambda cell: cell[1](), SELFTEST_CELLS))
    failures = [(label, problems, lines) for (label, _), (problems, lines) in zip(SELFTEST_CELLS, results) if problems]
    for label, problems, lines in failures:
        print(f"  ✗ make-swap-spec 自证：{label}：{'；'.join(problems)}；输出头一行 {lines[:1]}")  # gate-lint:detail
    if failures:
        print(f"  → 看 build_items() / file_items() / generate() / parse_arguments() / work_roots_now() / write_json_atomically() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ make-swap-spec 自证 {len(SELFTEST_CELLS)} 格都过（并行 {threads} 路，临时目录里造基线与新版）：普通改动出规格、正着倒着都核过；"
          "新串在施加之后出现两次的自动多带上下文；带到上限仍定不了位的拒并点名那一处；基线里没有点名文件的、新版不是 UTF-8 的、--out 不在工作根下的、--out 是目录或所在目录不在的、"
          "--max-context 不是正整数的、路径绝对或落到根外的、同一份点两次的、工作根取不到的、一处改动都没有的拒，都不写规格；"
          "命令前设 SINGLEFS_WORK_ROOT 或 SINGLEFS_MULTI_HOST_CONFIG 改不了工作根；--out 落在 git 工作树里（往上有 .git、经符号链接指进工作树）的拒；仓副本里起不认副本；事先放在临时文件名与 --out 上的符号链接不被写穿")
    return 0


USAGE = "make-swap-spec.py --base <基线根> --new <新版根> --out <规格.json> [--max-context <行数>] <相对路径>…，或 make-swap-spec.py --selftest"


def parse_arguments(arguments, mode=""):
    """→ (选项表, 相对路径表)；不对抛 Refused（退 2）。弄坏开关：max-context-unchecked 下不核 --max-context，path-escape-accepted 下不核路径是不是绝对、落没落到根外，
    duplicate-path-accepted 下同一份点两次也收。"""
    options, relatives, position = {"--base": None, "--new": None, "--out": None, "--max-context": str(MAX_CONTEXT_LINES)}, [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in options and position + 1 < len(arguments):
            options[argument] = arguments[position + 1]
            position += 2
            continue
        if argument.startswith("--"):
            raise Refused(EXIT_USAGE, f"✗ 不认的选项或缺值：{argument}\n→ 怎么办：{USAGE}")
        relatives.append(argument)
        position += 1
    if None in (options["--base"], options["--new"], options["--out"]) or not relatives:
        raise Refused(EXIT_USAGE, f"✗ 参数不对：--base、--new、--out 都要给，至少点名一份文件\n→ 怎么办：{USAGE}")
    value = options["--max-context"]
    if mode != "max-context-unchecked" and not (value.isascii() and value.isdigit() and int(value) >= 1):
        raise Refused(EXIT_USAGE, f"✗ --max-context {value} 不是正整数\n→ 怎么办：写前后各最多带几行上下文，省掉是 {MAX_CONTEXT_LINES}")
    normalized = [os.path.normpath(relative) for relative in relatives]
    if mode != "path-escape-accepted":
        for relative, written in zip(normalized, relatives):
            if os.path.isabs(written) or relative == ".." or relative.startswith(".." + os.sep) or relative == ".":
                raise Refused(EXIT_USAGE, f"✗ {written} 不是相对两个根、落在根里的路径\n→ 怎么办：写相对仓根的路径（例 scripts/gate/hook-swap.py），与规格里的 file 一样")
    if mode != "duplicate-path-accepted" and len(set(normalized)) != len(normalized):
        raise Refused(EXIT_USAGE, f"✗ 同一份文件点了两次：{' '.join(relatives)}\n→ 怎么办：每份只点一次")
    return options, normalized


def run(arguments, repository_root, environment):
    """一次出规格的全程 → (退出码, 提示行)：解析参数、取工作根、generate。弄坏开关取 break_mode()（平常跑设了在入口就被拒）。"""
    mode = break_mode()
    try:
        options, relatives = parse_arguments(arguments, mode)
        work_roots = work_roots_now(repository_root, environment, mode)
        return generate(options["--base"], options["--new"], options["--out"], relatives, int(options["--max-context"]), work_roots, mode)
    except Refused as refusal:
        return refusal.code, str(refusal).splitlines()


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    code, lines = run(arguments, REPOSITORY_ROOT, os.environ)
    print("\n".join(lines))
    return code


def refuse_break_outside_selftest(arguments):
    """弄坏开关（自己的 BREAK_VARIABLE 与工作根库的 WORK_ROOT_BREAK_VARIABLE）只在自证里认：设了而命令行不是单独一个 --selftest，点名变量、退 2；否则交 None。"""
    setting = [name for name in (BREAK_VARIABLE, WORK_ROOT_BREAK_VARIABLE) if os.environ.get(name)]
    if not setting or arguments == ["--selftest"]:
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一格，跑 {BREAK_VARIABLE}=<项> python3 scripts/gate/make-swap-spec.py --selftest", file=sys.stderr)
    return EXIT_USAGE


if __name__ == "__main__":
    refused = refuse_break_outside_selftest(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
