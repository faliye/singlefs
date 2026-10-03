import hashlib, importlib.util, subprocess, sys
root = sys.argv[1]
spec = importlib.util.spec_from_file_location("d", root + "/research/scripts/broad-mutation.py"); d = importlib.util.module_from_spec(spec); spec.loader.exec_module(d)
print("EXCLUSIONS_FILE =", d.EXCLUSIONS_FILE)
print("门禁第一格算全局键用的 exclusions_digest_of(EXCLUSIONS_FILE) =", d.exclusions_digest_of(d.EXCLUSIONS_FILE)[:16])
staged = subprocess.run(["git", "-C", root, "show", ":research/scripts/broad-mutation-ignored-exclusions.tsv"], capture_output=True).stdout
print("暂存区那一份的 sha256 =", hashlib.sha256(staged).hexdigest()[:16])
print("驱动第二段排除用例用的 load_exclusions() 条数 =", len(d.load_exclusions()) if hasattr(d, "load_exclusions") else "n/a")
