#!/usr/bin/env python3
"""模型 M6（gate-shrink-r1 云端攻方）：12 号 term-renames 那一格（research/scripts/sweep-term.py --check）慢在哪。
sweep-term.py 对每份文件先用 PRODUCT_FILE_NAME_PATTERN 把「已归档产物的文件名」换成占位符，再用改名检测正则数旧名。
这里对指定的几份文件分别计时：① 原样的 mask；② 只给同一个正则加上左边界（前一个字符不是 [A-Za-z0-9_.-]）的 mask；
③ 改名检测正则。并逐字比 ① 与 ② 换出来的文本、被换下来的原串是否相同（相同才说明改法不改判定）。
只读仓里的文件，不改 sweep-term.py。
用法：python3 m6_term_renames_mask_regex.py <仓根> <相对路径>…
"""
import importlib.util, os, re, sys, time

root = sys.argv[1]
sys.path.insert(0, os.path.join(root, "research/scripts"))
spec = importlib.util.spec_from_file_location("sweep_term", os.path.join(root, "research/scripts/sweep-term.py"))
sweep_term = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sweep_term)
archived_names = sweep_term.archived_product_file_names(root)
_replacements, detector = sweep_term.compile_rules(sweep_term.load_renames(root))
original_pattern = sweep_term.PRODUCT_FILE_NAME_PATTERN
bounded_pattern = re.compile(r"(?<![A-Za-z0-9_.-])" + original_pattern.pattern)


def mask_with(pattern, text):
    masked = []

    def to_placeholder(match):
        if match.group(0) not in archived_names:
            return match.group(0)
        masked.append(match.group(0))
        return "\x00archived%d\x00" % (len(masked) - 1)
    return pattern.sub(to_placeholder, text), masked


print(f"archived_names={len(archived_names)} original={original_pattern.pattern!r} bounded={bounded_pattern.pattern!r}")
for relative in sys.argv[2:]:
    text = open(os.path.join(root, relative), encoding="utf-8").read()
    start = time.monotonic(); original_text, original_masked = mask_with(original_pattern, text); original_seconds = time.monotonic() - start
    start = time.monotonic(); bounded_text, bounded_masked = mask_with(bounded_pattern, text); bounded_seconds = time.monotonic() - start
    start = time.monotonic(); hits = len(detector.findall(original_text)); detect_seconds = time.monotonic() - start
    longest_run = max((len(run) for run in re.findall(r"[A-Za-z0-9_.-]+", text)), default=0)
    print(f"{relative}\tchars={len(text)}\tlongest_[A-Za-z0-9_.-]_run={longest_run}\tmask_original={original_seconds:.2f}s\t"
          f"mask_bounded={bounded_seconds:.2f}s\tdetect={detect_seconds:.2f}s\tsame_text={original_text == bounded_text}\t"
          f"same_masked={original_masked == bounded_masked}\told_name_hits={hits}", flush=True)
