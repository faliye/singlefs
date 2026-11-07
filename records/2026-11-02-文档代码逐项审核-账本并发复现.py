"""Audit a controlled interleaving after both real digest checks, before rename.

Task-server only. Both appends use actual project functions and a temporary
ledger. The wrapper adds scheduling barriers; it does not bypass either check.
"""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
from pathlib import Path
import sys
import tempfile
import threading


def main():
    root = Path(__file__).resolve().parents[1]
    directory = root / "scripts" / "kb"
    sys.path.insert(0, str(directory))
    source = directory / "ledger-append-section.py"
    before = hashlib.sha256(source.read_bytes()).hexdigest()
    spec = importlib.util.spec_from_file_location("audit_ledger_race", source)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    if module.BROKEN:
        raise AssertionError("Mutation switch must be unset")
    real_replace = module.replace_file_contents_by_rename
    both_checked = threading.Barrier(2)
    first_finished = threading.Event()
    worker = threading.local()

    def controlled_replace(path, text, check_before_rename=None):
        def checked_then_wait():
            check_before_rename()
            print(f"AUDIT_LEDGER digest_check_passed writer={worker.label}", flush=True)
            both_checked.wait(timeout=15)
            if worker.label == "B" and not first_finished.wait(timeout=15):
                raise AssertionError("First writer did not finish")
        return real_replace(path, text, check_before_rename=checked_then_wait)

    module.replace_file_contents_by_rename = controlled_replace
    with tempfile.TemporaryDirectory(prefix="audit-ledger-race-") as scratch:
        fake_root, ledger = module.make_root(scratch, "repo", module.sample_ledger(["一"], ["一"]))

        def append(label):
            worker.label = label
            try:
                return module.append_section(ledger, "entry " + label, "body " + label + "\n",
                                             [("status", "open"), ("history", "none"), ("remaining", "all")],
                                             notation_root=fake_root)
            finally:
                if label == "A":
                    first_finished.set()

        with ThreadPoolExecutor(max_workers=2) as pool:
            first = pool.submit(append, "A")
            second = pool.submit(append, "B")
            codes = [first.result(), second.result()]
        final = Path(ledger).read_text()
        headings, indexed, _, _ = module.scan(final.splitlines())
        print(f"AUDIT_LEDGER exits={codes} body_A_present={'body A' in final} body_B_present={'body B' in final} headings={headings} indexed={indexed}", flush=True)
        if codes != [0, 0] or "body A" in final or "body B" not in final or headings != [1, 2] or indexed != [1, 2]:
            raise AssertionError("Expected lost update not observed; re-audit source")
    after = hashlib.sha256(source.read_bytes()).hexdigest()
    if before != after:
        raise AssertionError("Source changed during audit")
    print(f"AUDIT_LEDGER source_sha256={after} lost_update_reproduced=true temporary_ledger_removed=true", flush=True)


if __name__ == "__main__":
    main()
