"""判定库：按输入存判定，下一趟输入指纹与判法版本都相同就复用，不再起重活。
判法照 .claude/singlefs-ai-sop/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」。

Python 一侧这一个概念只叫「判定库」（类名 VerdictLibrary）。Rust 一侧 crates/singlefs-checker-tier 的判定存储（verdict_store 模块）是另一份存法，不在这里。
用它的两份：
  门禁 checker-tier-formal-verification 的两份判法（scripts/gate/verus-proof-check.py、scripts/experiments/e165-pin-red-examples.py）用整套：
    VerdictLibrary 存放与按键读、write_atomically 原子写、Tally 现判与复用分开报。
  门禁 checker-tier-crates-mutation-replay 的判法（scripts/mutation/crates-mutation-rows.py）用 write_atomically、read_text 与 Tally；
    它的判定库每条还挂覆盖记录、快照与源码副本，存放布局与复用判据写在它文件头「判定库」一段。
不用它、各自留一份的（gate-overlap:copy-kept 写在它们那里）：全绿标记（scripts/gate/admission.py，一道阶段或一格一个标记文件，在就复用）、
  广谱变异的结论表（scripts/mutation/broad-mutation.py，只追加的 TSV，按变异的函数键取最后一行）。

键分两列：
  输入指纹  这一项被判的全部输入（被判的代码、表里这一行、工具链、限时……）各自的摘要（components：[(名字, 摘要)]，名字是给人看的「核 crate」「这一行」这类）
            按名字排好，「名字 NUL 摘要 换行」接起来取 sha256。
  判法版本  判这一项的那份代码（判法脚本与它 import 的库，由调用方算好交进来）的摘要；一个判定库一个，建库时给。
  键 = sha256（「输入指纹 NUL 判法版本」）。两列有一样变了就现判；现判的原因点名是哪一列：输入里「<哪几样> 变了」，或「判法版本 变了」。
  只记绿：红、没跑成、被信号停了的一律不记，下一趟照判。键相同就复用，不看记下来的是哪一项的名字（输入里要有这一项自己的那一段，不然两项会共用一份判定）。
  一项判完当场落盘（write_atomically：同目录排他建临时文件、写完 fsync、改名换上）；被打断之后同一条命令重跑，判过的复用、没判的接着判。

存放（directory 由调用方给，门禁 checker-tier-formal-verification 给 git common-dir 下的 singlefs-formal-verdicts/，跨轮留着）：
  <directory>/<kind>/keys/<键>.json      这一份判定：item、key、input_fingerprint、judge_version、components、verdict（只有 green）、summary、recorded（日期）
  <directory>/<kind>/items/<项名的 sha256>.json   这一项最近一次判绿的那一份（与上面同内容），只用来说清「现判」的原因：哪一列、哪几样输入变了

用法：
    from lib_verdict_library import VerdictLibrary, Tally, digest_text, digest_file, digest_tree
    library = VerdictLibrary(目录或 None, "must-red-worlds", 判法版本, reuse=True)
    reused, reason = library.lookup(项名, components)
    ... 现判 ...
    library.record(项名, components, "一句判定")   # 只在判绿时调
    tally.count(reused, reason); print(tally.text())
目录给 None：不读不写（reason 写「没给判定库」），判法照旧全部现判。
这份只被 import，没有命令行入口；它的判法由两份形式化判法与 crates-mutation-rows.py 的自证覆盖（复用、关了复用、输入变了、判法版本变了、被打断续判、红的不记）。
"""
import datetime
import hashlib
import json
import os
import tempfile

NO_LIBRARY_REASON = "没给判定库"
REUSE_OFF_REASON = "关了复用"
NO_RECORD_REASON = "无记录"
BROKEN_RECORD_REASON = "记录读不出"
JUDGE_VERSION_COLUMN = "判法版本"


def digest_text(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape") if isinstance(text, str) else text).hexdigest()


def digest_file(path):
    """文件内容的 sha256；不在写成「没有」的摘要（不在也是一种取值）。"""
    if not os.path.isfile(path):
        return digest_text(f"没有 {os.path.basename(path)}")
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def digest_tree(directory, skipped_names=("target",), skipped_files=()):
    """目录下全部普通文件（跳过名叫 skipped_names 的子目录，与相对 directory 的路径在 skipped_files 里的文件）逐个「sha256  相对路径」
    按路径字节序接起来再取 sha256；目录不在写成「没有」的摘要。"""
    if not os.path.isdir(directory):
        return digest_text(f"没有目录 {os.path.basename(directory)}")
    skipped = {os.path.normpath(path) for path in skipped_files}
    lines = []
    for walked, subdirectories, names in os.walk(directory):
        subdirectories[:] = sorted(name for name in subdirectories if name not in skipped_names)
        for name in names:
            full = os.path.join(walked, name)
            if os.path.isfile(full) and os.path.relpath(full, directory) not in skipped:
                lines.append((os.path.relpath(full, directory).encode("utf-8", "surrogateescape"), digest_file(full)))
    return digest_text(b"".join(digest.encode() + b"  " + relative + b"\n" for relative, digest in sorted(lines)))


def input_fingerprint(components):
    """输入指纹：components 按名字排好，「名字 NUL 摘要 换行」接起来取 sha256。"""
    return digest_text("".join(f"{name}\0{digest}\n" for name, digest in sorted(components)))


def key_of(components, judge_version):
    """键 = sha256（「输入指纹 NUL 判法版本」）。"""
    return digest_text(f"{input_fingerprint(components)}\0{judge_version}")


def write_atomically(path, data):
    """同目录排他建临时文件、写完 fsync、改名换上；data 是 str 或 bytes，所在目录不在先建。失败时删掉临时文件，原文件不动，异常照抛。"""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    handle_number, temporary = tempfile.mkstemp(dir=os.path.dirname(path), prefix=os.path.basename(path) + ".partial.")
    try:
        if isinstance(data, bytes):
            with os.fdopen(handle_number, "wb") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
        else:
            with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
                handle.write(data)
                handle.flush()
                os.fsync(handle.fileno())
        os.replace(temporary, path)
    except BaseException:
        if os.path.exists(temporary):
            os.unlink(temporary)
        raise


def read_text(path):
    """按键读一份记录的原文：不在、读不了交 None（下一趟照判，不抛）。"""
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            return handle.read()
    except OSError:
        return None


def read_json(path):
    text = read_text(path)
    if text is None:
        return None
    try:
        return json.loads(text)
    except ValueError:
        return None


class VerdictLibrary:
    def __init__(self, directory, kind, judge_version, reuse=True):
        self.directory = os.path.join(directory, kind) if directory else None
        self.judge_version = judge_version
        self.reuse = reuse

    def _key_path(self, key):
        return os.path.join(self.directory, "keys", key + ".json")

    def _item_path(self, item):
        return os.path.join(self.directory, "items", digest_text(item) + ".json")

    def lookup(self, item, components):
        """交 (复用?, 原因)：复用时原因是记下那一份的日期与一句判定；现判时原因是没给判定库、关了复用、无记录、记录读不出，
        或「<输入里哪几样>、判法版本 变了」（变了的输入按名字排，判法版本变了排在最后）。"""
        if self.directory is None:
            return False, NO_LIBRARY_REASON
        if not self.reuse:
            return False, REUSE_OFF_REASON
        key = key_of(components, self.judge_version)
        if os.path.exists(self._key_path(key)):
            record = read_json(self._key_path(key))
            if record is None or record.get("verdict") != "green" or record.get("key") != key:
                return False, BROKEN_RECORD_REASON
            latest = read_json(self._item_path(item))
            if latest is None or latest.get("key") != key:
                # 这一项最近那一份换成复用的这一份：下一趟「现判」的原因比的是这一趟的输入
                try:
                    write_atomically(self._item_path(item), json.dumps(dict(record, item=item), ensure_ascii=False, sort_keys=True) + "\n")
                except OSError:
                    pass
            return True, f"{record.get('recorded', '?')} 判绿：{record.get('summary', '')}"
        latest = read_json(self._item_path(item))
        if latest is None or not isinstance(latest.get("components"), dict):
            return False, NO_RECORD_REASON
        changed = [name for name, digest in sorted(components) if latest["components"].get(name) != digest]
        changed += [name for name in sorted(latest["components"]) if name not in dict(components)]
        if latest.get("judge_version") != self.judge_version:
            changed.append(JUDGE_VERSION_COLUMN)
        return False, ("、".join(changed) + " 变了") if changed else BROKEN_RECORD_REASON

    def lookup_by_item_only(self, item):
        """弄坏开关用（两份判法的 reuse-ignores-key）：这一项最近判绿过就复用，不比键。正常判法不调它。"""
        latest = read_json(self._item_path(item))
        if latest is None:
            return False, NO_RECORD_REASON
        return True, f"{latest.get('recorded', '?')} 判绿（没比键）：{latest.get('summary', '')}"

    def record(self, item, components, summary):
        """判绿之后调：键那一份与这一项最近那一份各原子写一次。写不成交一句原因（不抛：后果只是下一趟这一项照判），写成交 None。"""
        if self.directory is None:
            return None
        key = key_of(components, self.judge_version)
        text = json.dumps({"item": item, "key": key, "input_fingerprint": input_fingerprint(components), "judge_version": self.judge_version,
                           "components": dict(components), "verdict": "green", "summary": summary,
                           "recorded": datetime.date.today().isoformat()}, ensure_ascii=False, sort_keys=True) + "\n"
        try:
            write_atomically(self._key_path(key), text)
            write_atomically(self._item_path(item), text)
        except OSError as error:
            return f"判定库 {self.directory} 写不进：{error}"
        return None


class Tally:
    """一格里现判几项、复用几项，现判的按原因分栏（形式化两份的原因是 lookup 交的那一句，crates 变异表的是复用判据的栏名）。"""

    def __init__(self):
        self.reused = 0
        self.reasons = {}

    def count(self, reused, reason):
        if reused:
            self.reused += 1
        else:
            self.reasons[reason] = self.reasons.get(reason, 0) + 1

    @property
    def judged(self):
        return sum(self.reasons.values())

    def text(self):
        columns = "、".join(f"{reason} {count}" for reason, count in sorted(self.reasons.items(), key=lambda pair: (-pair[1], pair[0])))
        return f"现判 {self.judged}、复用 {self.reused}" + (f"（现判的原因：{columns}）" if columns else "")

    def append_to(self, path, cell):
        """门禁给的 FORMAL_ITEM_TALLY_FILE：追加一行「格<TAB>现判数<TAB>复用数<TAB>原因分栏」，门禁汇总时整道相加。"""
        if not path:
            return
        columns = "、".join(f"{reason} {count}" for reason, count in sorted(self.reasons.items()))
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"{cell}\t{self.judged}\t{self.reused}\t{columns}\n")
