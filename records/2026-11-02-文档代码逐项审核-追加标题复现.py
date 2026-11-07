"""Task-server audit: actual append validators, pure inputs, no record writes."""
import hashlib
import importlib.util
from pathlib import Path
import sys


def load(path, name):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def main():
    root = Path(__file__).resolve().parents[1]
    directory = root / "scripts" / "kb"
    sys.path.insert(0, str(directory))
    paths = [directory / "append-record.py", directory / "ledger-append-section.py"]
    before = [hashlib.sha256(p.read_bytes()).hexdigest() for p in paths]
    append = load(paths[0], "audit_append_record")
    ledger = load(paths[1], "audit_ledger_append")
    if append.BROKEN or ledger.BROKEN:
        raise AssertionError("Audit requires all mutation switches unset")
    cases = [
        ("paragraph", "ordinary paragraph\n", False, False),
        ("heading", "## real heading\n", True, True),
        ("simple_fence", "```\n## code\n```\n", False, False),
        ("shorter_inner_fence", "````\n```\n## still code\n````\n", False, True),
        ("indented_heading", "  ## real heading\n", True, False),
        ("indented_code_then_heading", "    ```\n## real heading\n", True, False),
    ]
    functions = [
        ("append-record", append.text_problem),
        ("ledger-append-section", lambda body: ledger.body_problem("title", body, (("status", "open"),))),
    ]
    discrepancies = 0
    for name, function in functions:
        for label, body, should_refuse, observed_refusal in cases:
            problem = function(body)
            actual = problem is not None
            print(f"AUDIT_HEADING tool={name} case={label} expected_refusal={should_refuse} actual_refusal={actual} message={problem!r}", flush=True)
            if actual != observed_refusal:
                raise AssertionError("Source behavior changed; re-audit expected discrepancy")
            discrepancies += actual != should_refuse
    after = [hashlib.sha256(p.read_bytes()).hexdigest() for p in paths]
    if before != after:
        raise AssertionError("Source changed during audit")
    for path, digest in zip(paths, after):
        print(f"AUDIT_HEADING source={path.relative_to(root)} sha256={digest}", flush=True)
    print(f"AUDIT_HEADING scenarios=12 discrepancies={discrepancies} record_writes=0", flush=True)
    if discrepancies != 6:
        raise AssertionError(discrepancies)


if __name__ == "__main__":
    main()
