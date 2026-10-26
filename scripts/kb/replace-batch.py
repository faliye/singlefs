#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""批量定点替换：规格里的每一处都先在内存里核「恰好命中 count 次」（默认 1），全过了才写盘并回读。

用法：
  replace-batch.py 规格.json            # 核完再写
  replace-batch.py --dry-run 规格.json  # 只核不写
  replace-batch.py --selftest           # 自证会红

规格是一个 JSON 数组，每项 {"file": 路径, "old": 旧串, "new": 新串, "count": 可省，默认 1}；
同一个文件的多处按数组顺序在内存里依次施加，后一处的旧串按前一处改完的文本核。

为什么两阶段：逐处写盘的批量替换在第 k 处锚点不中时，前 k − 1 处已经落盘，留下半套改动。
2026-09-14 实测：一次 54 处的 kb 写回里有一处原文是「没被」而锚点写成「没有被」，两阶段让它一个文件都没写。
REPLACE_BATCH_WRITE_EACH=1 强制走回逐处写盘的旧行为，--selftest 在它下面必须判红。

写回不在原文件上就地写：同目录排他新建临时文件、fsync、照原权限位与属主、改名换上（`lib_atomic_replace.py`）。
正在按偏移边跑边读这份文件的进程（bash 跑脚本）读的仍是旧 inode 的旧内容。
换之前整批逐个文件预演一遍（临时文件写完、fsync 完，在改名之前停下、删掉）：有一个文件会被拒（多个硬链接、
当前用户只读、建不了临时文件、写或 fsync 失败），一个文件都不换，与「锚点不中一个文件都不写」同一条。

退出码：0 写成功（或 --dry-run 核过）；1 锚点命中数不对，一个文件都没写；2 用法不对；3 写完回读不一致；
4 没换上：预演时被拒，一个文件都没换；预演过了、真换时失败，已经换上的文件逐个列出；
5 目标在仓里（任何后缀），某一处的新串带会话号（ListAgents 列的会话名），一个文件都没写（--dry-run 同样判）；
6 目标是仓里的 .md，某一处的新串带钟点、时区词或时间戳（含运行目录名、日志文件名里连写的紧凑写法），一个文件都没写（--dry-run 同样判）。
核的时候每一处都核完（对不上的那一处不施加、接着核下一处），对不上的逐处列全再退 1（--dry-run 同样）：
命中 0 次的附上文件里最像的一段与第一处不同（replace-once.py 的 nearest_hint，同一份算法），文件读不了的点名那个文件。

写仓里的文件之前判每一处的新串里的会话号（退出码 5）：锚点都核对得上之后、写之前（--dry-run 也判），用 write-guard.sh 的第七道与门禁 doc-text 的 session-names 格同一份判据
（.claude/gate.d/lib-forbidden-notations.py 的会话号形态，lib_notation_check.py 的 session_name_refusal 调它，三份定点写入脚本共用）判规格里每一处的 new；
带会话名的逐处列出文件、行号与出路（写成「别的会话」这类不带号的说法）。后缀不限，范围与写闸判 Write、Edit 一致：排除前缀、.sh .py .rs 里带
「# session-names:allow <理由>」的一行、仓外不判。拦不住的写法（shell 重定向、heredoc、python 的 open）靠提交时门禁 doc-text 的 session-names 格兜底。
写仓里的 .md 之前判每一处的新串里的钟点（退出码 6）：锚点都核对得上之后、判会话号之前（--dry-run 也判），用 write-guard.sh 的第五道与门禁 doc-text 的 clock-times 格
同一份判据（.claude/gate.d/lib-forbidden-notations.py 的钟点形态，lib_notation_check.py 的 clock_refusal 调它，与 replace-once.py、insert-row.py 同一份）判规格里每一处的 new；
带钟点、时区词、时间戳的逐处列出文件、行号、形态与出路。判新串不判旧串；别的后缀、仓外、排除前缀下的文件不判；读不到判据库按拒绝办（退出码 6）。

弄坏开关（只给 --selftest 证红用）：
  REPLACE_BATCH_BREAK=stop-at-first  plan_in_memory() 第一处对不上就停（改前的判法）
      一次核完（「✗ 自证：四处里第 1、3、4 处对不上」，失败行下面原样转述 run() 的输出：「✗ 第 1 处 …third.md：期望命中 1 次，实际 0」「✗ 4 处里 1 处对不上」）
  REPLACE_BATCH_BREAK=no-clock-guard  run() 不判新串里的钟点（改前没有这一道）
      写 .md 之前判新串里的钟点（「✗ 自证：写 .md 之前判新串里的钟点没判对」，下面逐条以「写 .md：」开头：新串带紧凑时间戳、带日期钟点时区词、带光秃时区词、
      --dry-run 也拒、两处里一处带钟点时一个文件都不写、入口退出码 6）
  REPLACE_BATCH_BREAK=no-session-guard  run() 不判新串里的会话号（改前的写法）
      写之前判新串里的会话号（「✗ 自证：写会话号之前判新串没判对」，下面逐条以「写会话号：」开头：.md 与 .py 目标的新串带会话名、--dry-run 也拒、入口退出码 5 与不写文件）
  LIB_FORBIDDEN_NOTATIONS_BREAK=session-names  共用库的弄坏开关（写法在 lib-forbidden-notations.py 文件头），自证同样走它：整个会话号形态不判
      写之前判新串里的会话号（「✗ 自证：写会话号之前判新串没判对」，下面「写会话号：」开头的拒绝几条全红）；
      同一个库另有 LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names（放行标记不认：只有带放行标记的那一条红）与
      LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=session-names（排除前缀不生效：只有排除前缀下放行的那一条红），失败行同上
  LIB_FORBIDDEN_NOTATIONS_BREAK=clock-times  同一个库的钟点形态整个不判，自证同样走它
      写 .md 之前判新串里的钟点（「✗ 自证：写 .md 之前判新串里的钟点没判对」，下面「写 .md：」开头的拒绝几条全红）
  replace-once.py 的 REPLACE_ONCE_BREAK=no-nearest 同样让这一格红（最像的一段是从那里取的）
"""
import contextlib
import importlib.util
import io
import json
import os
import pathlib
import sys
import tempfile

from lib_atomic_replace import ReplaceRefused, leftover_temporary_files, problems_with_replacement_by_rename, \
    replace_file_contents_by_rename
from lib_notation_check import clock_refusal, copy_library_into, session_guard_problems, session_name_refusal
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402


def replace_once_module():
    """replace-once.py 当模块读进来：命中 0 次时找「文件里最像的一段」只有它的 nearest_hint 一份。"""
    spec = importlib.util.spec_from_file_location("replace_once", os.path.join(os.path.dirname(os.path.realpath(__file__)), "replace-once.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def plan_in_memory(edits):
    """每一处都核完（对不上的不施加、接着核下一处），→ (各文件改完的文本, [对不上的每一处的说明])；说明表空着才算全过。"""
    texts, errors, unreadable, nearest_hint = {}, [], {}, None
    for index, edit in enumerate(edits, 1):
        path = edit.get("file", "")
        expected_hits = edit.get("count", 1)
        if path in unreadable:
            errors.append(f"第 {index} 处 {path}：{unreadable[path]}")
            continue
        if path not in texts:
            try:
                texts[path] = pathlib.Path(path).read_text(encoding="utf-8")
            except (OSError, UnicodeDecodeError) as error:
                unreadable[path] = f"读不了这个文件（{error.__class__.__name__}：{error}）"
                errors.append(f"第 {index} 处 {path}：{unreadable[path]}")
                continue
        hits = texts[path].count(edit["old"])
        if hits != expected_hits:
            detail = f"第 {index} 处 {path}：期望命中 {expected_hits} 次，实际 {hits}：{edit['old'][:60]!r}"
            if hits == 0:
                nearest_hint = nearest_hint or replace_once_module().nearest_hint
                detail += "".join(f"\n  {line.strip()}" for line in nearest_hint(texts[path], edit["old"]))
            errors.append(detail)
            if os.environ.get("REPLACE_BATCH_BREAK") == "stop-at-first":   # 弄坏开关：改前的判法，第一处对不上就停
                return None, errors
            continue
        texts[path] = texts[path].replace(edit["old"], edit["new"])
    return (None if errors else texts), errors


def clock_refusals(edits, notation_root=None):
    """规格里每一处新串的钟点判定（共用库 clock-times 那一形态，写闸与 replace-once.py、insert-row.py 同一份）：→ [说明行列表]，每个有命中的新串一个。
    只判仓里的 .md 目标（clock_refusal 自己管后缀、仓外与排除前缀）；「第 n 行」的算法同 session_name_refusals。弄坏开关 no-clock-guard 不判。"""
    if os.environ.get("REPLACE_BATCH_BREAK") == "no-clock-guard":
        return []
    texts, refusals = {}, []
    for edit in edits:
        path = edit["file"]
        if path not in texts:
            texts[path] = pathlib.Path(path).read_text(encoding="utf-8")
        refusal = clock_refusal(path, edit["new"], texts[path][:texts[path].find(edit["old"])].count("\n") + 1, notation_root)
        if refusal:
            refusals.append(refusal)
        texts[path] = texts[path].replace(edit["old"], edit["new"])
    return refusals


def session_name_refusals(edits, notation_root=None):
    """规格里每一处新串的会话号判定（共用库 session-names 那一形态，写闸同一份）：→ [说明行列表]，每个有命中的新串一个。
    各处按数组次序在内存里依次施加，「第 n 行」是那一处改完之后新串第一行在文件里的行号。弄坏开关 no-session-guard 不判。"""
    if os.environ.get("REPLACE_BATCH_BREAK") == "no-session-guard":
        return []
    texts, refusals = {}, []
    for edit in edits:
        path = edit["file"]
        if path not in texts:
            texts[path] = pathlib.Path(path).read_text(encoding="utf-8")
        refusal = session_name_refusal(path, edit["new"], texts[path][:texts[path].find(edit["old"])].count("\n") + 1, notation_root)
        if refusal:
            refusals.append(refusal)
        texts[path] = texts[path].replace(edit["old"], edit["new"])
    return refusals


def apply_each_edit_to_disk(edits):
    """旧行为：逐处写盘。只给自证强制走一遍，证明它会留下半套改动。"""
    for index, edit in enumerate(edits, 1):
        path = pathlib.Path(edit["file"])
        text = path.read_text(encoding="utf-8")
        expected_hits = edit.get("count", 1)
        hits = text.count(edit["old"])
        if hits != expected_hits:
            return f"第 {index} 处 {edit['file']}：期望命中 {expected_hits} 次，实际 {hits}"
        path.write_text(text.replace(edit["old"], edit["new"]), encoding="utf-8")
    return None


class RehearsalStoppedBeforeRename(Exception):
    """预演：临时文件写完、fsync 完，在改名换上之前停下；库删掉临时文件后把它原样抛出。"""


def rehearse_replacement(path, text):
    """走一遍整条换法、停在改名之前：拒绝与写临时文件时的失败在这里抛 ReplaceRefused，原文件一个字节没动。"""
    def stop_before_rename():
        raise RehearsalStoppedBeforeRename()
    try:
        replace_file_contents_by_rename(path, text, check_before_rename=stop_before_rename)
    except RehearsalStoppedBeforeRename:
        return


def replace_planned_texts(texts):
    """先整批预演，全过了再逐个改名换上。返回退出码；0 以外都已打印原因与下一步。"""
    for path, text in texts.items():
        try:
            rehearse_replacement(path, text)
        except ReplaceRefused as refused:
            print(f"✗ {refused}")
            print(f"→ 一个文件都没换（{len(texts)} 个文件逐个预演过，这一个会被拒）：处理完这个文件再跑同一份规格")
            return 4
    replaced_paths = []
    for path, text in texts.items():
        try:
            notes = replace_file_contents_by_rename(path, text)
        except ReplaceRefused as refused:
            print(f"✗ {refused}")
            print(f"→ 预演过了、真换时失败，这一批留了半套：已经换上的 {len(replaced_paths)} 个是 {replaced_paths}；"
                  "先核这几个的内容，再只拿剩下的几处重跑")
            return 4
        replaced_paths.append(path)
        for note in notes:
            print(f"! {note}")
    return 0


def run(edits, dry_run, notation_root=None):
    if os.environ.get("REPLACE_BATCH_WRITE_EACH") == "1":
        error = apply_each_edit_to_disk(edits)
        if error:
            print(f"✗ {error}")
            print("→ 对一下原文再跑（这是逐处写盘的旧行为，前面几处已经落盘了）")
            return 1
        return 0
    texts, errors = plan_in_memory(edits)
    if errors:
        for error in errors:
            print(f"✗ {error}")  # gate-lint:detail
        print(f"✗ {len(edits)} 处里 {len(errors)} 处对不上（逐处列在上面{'' if len(errors) == 1 else '；同一个文件里排在对不上那一处后面的，是按没施加它的文本核的'}）"
              f"{'，--dry-run 没写' if dry_run else '，什么都没写'}")  # gate-lint:summary
        print("→ 怎么办：对一下每一处的原文（空白、全半角、同义字；命中 0 次的看上面「最像的一段」与第一处不同），改规格再整份重跑")
        return 1
    clock_hits = clock_refusals(edits, notation_root)
    if clock_hits:
        for refusal in clock_hits:
            print("✗ " + "\n".join(refusal))  # gate-lint:detail
        print(f"✗ {len(edits)} 处里 {len(clock_hits)} 处的新串在描述里写了钟点、时区词或时间戳{'，--dry-run 没写' if dry_run else '，什么都没写'}")  # gate-lint:summary
        print("→ 怎么办：照上面每一处的出路改新串（钟点与时区词去掉、只写日期，运行目录名与日志文件名里的时间戳不抄进 .md），改规格整份重跑")
        return 6
    refusals = session_name_refusals(edits, notation_root)
    if refusals:
        for refusal in refusals:
            print("✗ " + "\n".join(refusal))  # gate-lint:detail
        print(f"✗ {len(edits)} 处里 {len(refusals)} 处的新串写了会话号{'，--dry-run 没写' if dry_run else '，什么都没写'}")  # gate-lint:summary
        print("→ 怎么办：新串里的会话号换成「别的会话」这类不带号的说法，事情与日期照留，改规格整份重跑")
        return 5
    if dry_run:
        print(f"✓ {len(edits)} 处都命中得对（{len(texts)} 个文件），--dry-run 没写")
        return 0
    exit_code = replace_planned_texts(texts)
    if exit_code != 0:
        return exit_code
    for path, text in texts.items():
        if pathlib.Path(path).read_text(encoding="utf-8") != text:
            print(f"✗ {path} 回读与内存里的结果不一致")
            print("→ 看是不是有别的进程同时在写这个文件，核一遍再跑")
            return 3
    print(f"✓ {len(edits)} 处替换，写了 {len(texts)} 个文件（改名换上新 inode），回读一致")
    return 0


def problems_with_clock_guard(directory):
    """写 .md 之前判新串里的钟点、时区词、时间戳。样本串在这里拆开拼，这份源码里不出现字面写法（门禁 doc-text 也扫它）。
    仓根用自证的临时目录顶替（notation_root）：假仓里抄一份真的共用库（copy_library_into）；另造一个没有库的假仓核「读不到判据」那一支。"""
    problems = []
    glue = "".join
    colon = ":"
    zone = glue(("J", "ST"))
    stamp = glue(("20261014", "T153000Z"))
    root = os.path.join(directory, "clock-repo")
    os.makedirs(os.path.join(root, "records"))
    copy_library_into(root)
    notes = os.path.join(root, "records", "notes.md")
    other = os.path.join(root, "records", "other.md")
    original = "第一行 甲\n第二行 乙\n"

    def reset(path=notes):
        pathlib.Path(path).write_text(original, encoding="utf-8")

    def content_of(path=notes):
        return pathlib.Path(path).read_text(encoding="utf-8")

    def attempt(batch, dry_run=False, base=root):
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = run(batch, dry_run, notation_root=base)
        return code, captured.getvalue()

    reset()
    refused = [
        ("新串带紧凑时间戳", glue(("运行目录 ", stamp)), "（ISO 紧凑时间戳）：「" + stamp + "」"),
        ("新串带日期、时区词、时分", glue(("用户 2026-10-14 ", zone, " 15", colon, "30 定")), "（日期后面跟钟点）"),
        ("新串带光秃的时区词", glue(("本机时钟是 ", zone)), "（时区词）：「" + zone + "」"),
    ]
    for label, new_text, wanted in refused:
        for dry_run in (False, True):
            code, said = attempt([{"file": notes, "old": "第二行 乙", "new": new_text}], dry_run)
            if code != 6 or wanted not in said or "第 2 行" not in said or "→ 怎么办" not in said or "读不到钟点判据" in said:
                problems.append(f"写 .md：{label}{'（干跑）' if dry_run else ''}本该退出码 6、点名形态「{wanted}」与第 2 行与出路，实际 {code}：{said!r}"
                                "（REPLACE_BATCH_BREAK=no-clock-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK 关掉那一形态的话这一条本来就该红）")
            if content_of() != original or sorted(os.listdir(os.path.dirname(notes))) != ["notes.md"]:
                problems.append(f"写 .md：{label}被拒绝之后文件或目录变了（一个字节都不该写、不该留临时文件）")
    # 两处里一处带钟点：两个文件都不写
    reset()
    reset(other)
    code, said = attempt([{"file": other, "old": "第二行 乙", "new": "第二行 丙"}, {"file": notes, "old": "第二行 乙", "new": glue(("运行目录 ", stamp))}])
    if code != 6 or "2 处里 1 处的新串在描述里写了钟点" not in said or content_of() != original or content_of(other) != original:
        problems.append(f"写 .md：两处里一处带钟点本该退出码 6、报「2 处里 1 处」且两个文件都不写，实际 {code}：{said!r}")
    os.remove(other)
    # 放行：8 位数字后跟 T 但不是时间戳、只写日期、旧串带钟点而新串干净、别的后缀与仓外不判
    reset()
    code, said = attempt([{"file": notes, "old": "第二行 乙", "new": glue(("编号 99999999", "T1234"))}])
    if code != 0 or content_of() != "第一行 甲\n编号 99999999" + "T1234\n":
        problems.append(f"写 .md：8 位数字后跟 T 但不是时间戳的新串本该放行（退出码 0），实际 {code}：{said!r}"
                        "（LIB_FORBIDDEN_NOTATIONS_BREAK=compact-timestamp-loose 设着的话这一条本来就该红）")
    reset()
    code, said = attempt([{"file": notes, "old": "第二行 乙", "new": "第二行 用户 2026-10-14 定"}])
    if code != 0 or content_of() != "第一行 甲\n第二行 用户 2026-10-14 定\n":
        problems.append(f"写 .md：只写日期的新串本该放行（退出码 0），实际 {code}：{said!r}")
    dirty = os.path.join(root, "records", "dirty.md")
    pathlib.Path(dirty).write_text(glue(("用户 2026-10-14 ", zone, " 15", colon, "30 定\n")), encoding="utf-8")
    code, said = attempt([{"file": dirty, "old": glue(("2026-10-14 ", zone, " 15", colon, "30 定")), "new": "2026-10-14 定"}])
    if code != 0 or content_of(dirty) != "用户 2026-10-14 定\n":
        problems.append(f"写 .md：旧串带钟点、新串只写日期（删掉钟点的改动）本该放行，实际 {code}：{said!r}")
    for label, path in (("仓里的 .txt 目标", os.path.join(root, "records", "notes.txt")), ("仓外的 .md", os.path.join(directory, "outside-clock.md"))):
        reset(path)
        code, said = attempt([{"file": path, "old": "第二行 乙", "new": glue(("运行目录 ", stamp))}])
        if code != 0:
            problems.append(f"写 .md：{label}不归这一道判，本该放行（退出码 0），实际 {code}：{said!r}")
    # 读不到共用库按拒绝办，不静默放行：假仓里没有库，新串干净也拒，理由是读不到钟点判据（目标是 .md，拒的是钟点这一道）
    bare = os.path.join(directory, "clock-repo-without-library")
    os.makedirs(os.path.join(bare, "records"))
    bare_notes = os.path.join(bare, "records", "notes.md")
    reset(bare_notes)
    code, said = attempt([{"file": bare_notes, "old": "第二行 乙", "new": "第二行 丙"}], base=bare)
    if code != 6 or "读不到钟点判据" not in said or "lib-forbidden-notations.py" not in said or "FileNotFoundError" not in said \
            or "→ 怎么办" not in said or content_of(bare_notes) != original:
        problems.append(f"写 .md：假仓里没有共用库，本该退出码 6、说「读不到钟点判据」点名库文件与 FileNotFoundError 与出路且不写文件，实际 {code}：{said!r}")
    # 入口：退出码与输出
    reset()
    specification = pathlib.Path(directory, "clock-spec.json")
    specification.write_text(json.dumps([{"file": notes, "old": "第二行 乙", "new": glue(("运行目录 ", stamp))}], ensure_ascii=False), encoding="utf-8")
    captured = io.StringIO()
    with contextlib.redirect_stdout(captured):
        code = main([str(specification)], notation_root=root)
    if code != 6 or not captured.getvalue().startswith("✗ 新串在描述里写了钟点、时区词或时间戳，共 1 处") or content_of() != original:
        problems.append(f"写 .md：入口本该退出码 6、打 ✗ 开头的说明且不写文件，实际 {code}：{captured.getvalue()!r}")
    return problems


def problems_with_session_guard(directory):
    """写之前判新串里的会话号：样本与判法在 lib_notation_check.py 的 session_guard_problems（三份脚本共用），这里只接上 run 与入口。"""
    def attempt(path, new_text, root, dry_run):
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = run([{"file": path, "old": "第二行 乙", "new": new_text}], dry_run, notation_root=root)
        return code, captured.getvalue()

    def entry(path, new_text, root):
        specification = pathlib.Path(directory, "session-spec.json")
        specification.write_text(json.dumps([{"file": path, "old": "第二行 乙", "new": new_text}], ensure_ascii=False), encoding="utf-8")
        captured = io.StringIO()
        with contextlib.redirect_stdout(captured):
            code = main([str(specification)], notation_root=root)
        return code, captured.getvalue()
    return session_guard_problems(directory, attempt, 5, entry)


def selftest():
    with tempfile.TemporaryDirectory() as directory:
        first = pathlib.Path(directory, "first.md")
        second = pathlib.Path(directory, "second.md")
        first.write_text("甲乙丙\n", encoding="utf-8")
        second.write_text("子丑寅\n", encoding="utf-8")
        before = (first.read_text(encoding="utf-8"), second.read_text(encoding="utf-8"))
        broken_batch = [{"file": str(first), "old": "乙", "new": "B"}, {"file": str(second), "old": "卯", "new": "M"}]
        exit_code = run(broken_batch, dry_run=False)
        after = (first.read_text(encoding="utf-8"), second.read_text(encoding="utf-8"))
        if exit_code == 0 or after != before:
            print("✗ 自证：第二处锚点不中时第一处已经落盘（或退出码是 0）——两阶段没有生效")
            print("→ 看 run() 是不是逐处写盘了；REPLACE_BATCH_WRITE_EACH=1 就是故意走这条路")
            return 1
        good_batch = [{"file": str(first), "old": "乙", "new": "B"}, {"file": str(first), "old": "甲B", "new": "AB"},
                      {"file": str(second), "old": "丑", "new": "C", "count": 1}]
        if run(good_batch, dry_run=False) != 0 or first.read_text(encoding="utf-8") != "AB丙\n" or second.read_text(encoding="utf-8") != "子C寅\n":
            print("✗ 自证：全部命中的一批没有按顺序写对")
            print("→ 看同一个文件的多处是不是按数组顺序在内存里依次施加")
            return 1
        if run([{"file": str(second), "old": "子", "new": "Z", "count": 2}], dry_run=False) == 0:
            print("✗ 自证：期望命中 2 次、实际 1 次也放行了")
            print("→ 看 plan_in_memory() 是不是按 count 比命中数")
            return 1
        # 一次核完：三处里第 1、3 处对不上（命中 0 次的附最像的一行），--dry-run 与真跑都逐处列全、一个文件都不写
        third = pathlib.Path(directory, "third.md")
        third.write_text("第一行 甲\n第二行 乙\n", encoding="utf-8")
        before = (first.read_text(encoding="utf-8"), third.read_text(encoding="utf-8"))
        batch = [{"file": str(third), "old": "第二行 丙", "new": "X"}, {"file": str(first), "old": "AB", "new": "ab"},
                 {"file": str(first), "old": "丙", "new": "C", "count": 2}, {"file": str(pathlib.Path(directory, "nothere.md")), "old": "a", "new": "b"}]
        for dry_run in (True, False):
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                exit_code = run(batch, dry_run=dry_run)
            said = captured.getvalue()
            wanted = ["第 1 处", "最像的一段在第 2 行", "旧串是「丙」，文件里是「乙」", "第 3 处", "期望命中 2 次，实际 1", "第 4 处", "读不了这个文件",
                      "4 处里 3 处对不上"]
            if exit_code != 1 or any(piece not in said for piece in wanted) or "第 2 处" in said \
                    or (first.read_text(encoding="utf-8"), third.read_text(encoding="utf-8")) != before:
                print(f"✗ 自证：四处里第 1、3、4 处对不上，{'--dry-run' if dry_run else '真跑'}应当退 1、逐处列全（命中 0 次的附最像的一行）、不列对得上的第 2 处、"
                      f"一个文件都不写，实际退 {exit_code}：\n{said}")
                print("→ 看 plan_in_memory() 是不是核完每一处再判；REPLACE_BATCH_BREAK=stop-at-first 设着的话这里本来就该红")
                return 1
    # 写回的方式：inode 要换、在读的进程读旧内容、权限位与符号链接保住、硬链接与只读拒绝、失败不留临时文件（判据在 lib_atomic_replace.py）
    running_script = "#!/usr/bin/env bash\n" + "echo 前面这几行正在被 bash 边跑边读\n" * 20 + "run_step 甲\nexit $?\n"
    problems = problems_with_replacement_by_rename(
        lambda path: run([{"file": path, "old": "run_step 甲", "new": "run_step 戊"}], dry_run=False),
        running_script, running_script.replace("run_step 甲", "run_step 戊", 1))
    if problems:
        print("✗ 自证：写回的方式不对")
        for problem in problems:
            print(f"   {problem}")  # gate-lint:detail
        print("→ 按方括号里那一格查 lib_atomic_replace.py 的 replace_file_contents_by_rename，"
              "以及 replace_planned_texts() 是不是还有就地写的那一步")
        return 1
    # 写 .md 之前判新串里的钟点
    with tempfile.TemporaryDirectory() as directory:
        clock_problems = problems_with_clock_guard(directory)
        if clock_problems:
            print("✗ 自证：写 .md 之前判新串里的钟点没判对")
            for problem in clock_problems:
                print(f"   {problem}")  # gate-lint:detail
            print("→ 看 run() 里调 clock_refusals 的那一段与 lib_notation_check.py；REPLACE_BATCH_BREAK=no-clock-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK 设着的话这里本来就该红")
            return 1
    # 写之前判新串里的会话号
    with tempfile.TemporaryDirectory() as directory:
        session_problems = problems_with_session_guard(directory)
        if session_problems:
            print("✗ 自证：写会话号之前判新串没判对")
            for problem in session_problems:
                print(f"   {problem}")  # gate-lint:detail
            print("→ 看 run() 里调 session_name_refusals 的那一段与 lib_notation_check.py；REPLACE_BATCH_BREAK=no-session-guard 或 LIB_FORBIDDEN_NOTATIONS_BREAK 设着的话这里本来就该红")
            return 1
    # 整批预演：第二个文件有两个硬链接会被拒，第一个文件也不许先换上
    with tempfile.TemporaryDirectory() as directory:
        first = pathlib.Path(directory, "first.md")
        shared = pathlib.Path(directory, "shared.md")
        first.write_text("甲乙丙\n", encoding="utf-8")
        shared.write_text("子丑寅\n", encoding="utf-8")
        os.link(shared, pathlib.Path(directory, "other-name.md"))
        inode_before = first.stat().st_ino
        batch = [{"file": str(first), "old": "乙", "new": "B"}, {"file": str(shared), "old": "丑", "new": "C"}]
        exit_code = run(batch, dry_run=False)
        if exit_code != 4 or first.read_text(encoding="utf-8") != "甲乙丙\n" or first.stat().st_ino != inode_before \
                or shared.read_text(encoding="utf-8") != "子丑寅\n" or leftover_temporary_files(directory):
            print(f"✗ 自证：批里第二个文件会被拒（两个硬链接），退出码 {exit_code}（该是 4），"
                  "第一个文件被先换上了、或留了临时文件——这一批留了半套")
            print("→ 看 replace_planned_texts() 是不是先把整批逐个预演完、全过了才开始改名换上")
            return 1
    print("✓ replace-batch 自证：锚点不中时一个文件都不写，全中时按序写对，命中数不等就拒绝，几处对不上时（--dry-run 同样）一次逐处列全、命中 0 次的附最像的一行；"
          "换上的是新 inode、权限位不变、改之前打开文件的读者接着读到旧内容、fsync 在改名之前、"
          "经符号链接改的是它指向的文件、有两个硬链接或当前用户只读都拒绝、fsync 或改名失败都不动原文件也不留临时文件；"
          "批里有一个文件会被拒时一个文件都不换；"
          "写仓里的 .md 之前判每一处新串里的钟点：带紧凑时间戳、日期钟点时区词、光秃时区词的拒绝（6，--dry-run 也拒、两处里一处带钟点一个文件都不写）并点名形态与行号，"
          "非时间戳的 8 位数字加 T、只写日期、删掉钟点的改动、别的后缀与仓外放行，假仓里没有共用库时读不到钟点判据按拒绝办，入口退出码 6；"
          "写仓里任何后缀的文件之前判每一处新串里的会话号：.md 与 .py 目标带会话名的拒绝（5，--dry-run 也拒）并点名行号与出路「别的会话」且一个文件都不写，干净的新串、.py 里带放行标记的一行、"
          "排除前缀下的文件、仓外的文件放行，假仓里没有共用库时读不到会话号判据按拒绝办（退出码 5），入口退出码 5")
    return 0


def main(arguments, notation_root=None):
    if arguments == ["--selftest"]:
        return selftest()
    dry_run = "--dry-run" in arguments
    positional = [argument for argument in arguments if argument != "--dry-run"]
    if len(positional) != 1:
        print("✗ 用法：replace-batch.py [--dry-run] 规格.json，或 replace-batch.py --selftest")
        print("→ 规格是 JSON 数组，每项 {\"file\", \"old\", \"new\", \"count\"（可省，默认 1）}")
        return 2
    try:
        edits = json.loads(pathlib.Path(positional[0]).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, ValueError) as error:
        print(f"✗ 读不了规格 {positional[0]}（{error.__class__.__name__}：{error}）")
        print("→ 怎么办：核对路径；规格是 JSON 数组，每项 {\"file\", \"old\", \"new\", \"count\"（可省，默认 1）}")
        return 2
    if not isinstance(edits, list) or not all(isinstance(edit, dict) and isinstance(edit.get("old"), str) and isinstance(edit.get("new"), str)
                                              and isinstance(edit.get("file"), str) for edit in edits):
        print(f"✗ 规格 {positional[0]} 不是 JSON 数组、或有一项缺 file / old / new（都要是字符串）")
        print("→ 怎么办：规格是 JSON 数组，每项 {\"file\", \"old\", \"new\", \"count\"（可省，默认 1）}")
        return 2
    return run(edits, dry_run, notation_root)


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
