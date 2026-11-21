"""crates-split-pure-move.py 的几个模块共用的东西：弄坏开关的登记与当前打开的集合、用法错、按字节读文本。
不单独起；弄坏开关只在那份脚本的 --selftest 里认（文件头「弄坏开关」），环境变量由那份入口读，这里不读。"""
import collections

# 开关名到它打开后红的全部自证格（按 CELLS 的次序）：开关轮与复跑格都核红的格集合与这里逐格相同，多红、少红都不对；
# 点名多于一格的，多出的是开关弄坏的那一处连带波及的格（别的格拿那一处的输出当前提）
BREAK_CELLS = collections.OrderedDict([
    ("visibility-not-stripped", ("visibility-relaxed-green", "scaffold-green", "inline-module-to-file-green", "visibility-counts", "removed-use-green", "exit-codes", "rustfmt-only-counts")),
    ("changed-ignored", ("changed-body-red", "crate-filter", "wgsl-changed-red", "exit-codes", "rustfmt-real-changes-red", "unchanged-files-silent")),
    ("missing-ignored", ("missing-unit-red", "duplicate-dropped-red", "duplicate-twin-unchanged-red", "removed-use-strict-red", "inline-module-attribute-dropped-red")),
    ("extra-ignored", ("extra-unit-red", "module-attributes-head-added-red")),
    ("scaffold-counted-as-extra", ("visibility-relaxed-green", "scaffold-green", "inline-module-to-file-green", "visibility-counts", "removed-use-green", "split-impl-green", "exit-codes", "unchanged-files-silent", "new-mod-heads-scaffold")),
    ("module-scope-kept-in-names", ("visibility-relaxed-green", "scaffold-green", "inline-module-to-file-green", "visibility-counts", "changed-body-red", "removed-use-green", "exit-codes")),
    ("duplicates-collapsed", ("duplicate-dropped-red", "split-impl-green", "new-mod-heads-scaffold")),
    ("strict-use-not-red", ("removed-use-strict-red",)),
    ("visibility-counted-as-identical", ("visibility-counts", "rustfmt-only-counts")),
    ("split-impl-header-extra", ("split-impl-green",)),
    ("inline-module-attributes-unchecked", ("inline-module-attribute-dropped-red",)),
    ("line-numbers-in-names", ("visibility-relaxed-green", "scaffold-green", "inline-module-to-file-green", "visibility-counts", "removed-use-green", "macro-call-moved-green", "exit-codes")),
    ("crate-filter-ignored", ("crate-filter",)),
    ("lone-crate-ignored", ("crate-missing-after-red",)),
    ("wgsl-skipped", ("wgsl-changed-red",)),
    ("red-exits-zero", ("changed-body-red", "missing-unit-red", "extra-unit-red", "duplicate-dropped-red", "duplicate-twin-unchanged-red", "removed-use-strict-red", "inline-module-attribute-dropped-red", "crate-filter", "crate-missing-after-red", "wgsl-changed-red", "unknown-item-changed-red", "exit-codes", "rustfmt-real-changes-red", "unchanged-files-silent", "module-attributes-head-added-red", "registry-path-boundary", "registry-test-names", "target-renames-registry")),
    ("refusals-unchecked", ("unknown-item-changed-red",)),
    ("ambiguous-first-hit-taken", ("anchor-in-two-files-listed", "anchor-staying-in-root-unchanged", "remap-counts")),
    ("repeated-hit-in-one-file-accepted", ("anchor-twice-in-one-file-listed", "anchor-staying-in-root-unchanged", "remap-counts")),
    ("zero-hit-dropped", ("anchor-nowhere-listed", "remap-counts")),
    ("newline-escape-undecoded", ("multi-line-anchor-changed", "only-the-file-column-changes", "remap-counts", "remap-exit-codes")),
    ("unrelated-files-searched", ("anchor-nowhere-listed", "anchor-only-in-split-family", "only-the-file-column-changes", "remap-counts", "crate-root-siblings-searched")),
    ("family-root-excluded", ("anchor-in-two-files-listed", "anchor-twice-in-one-file-listed", "anchor-staying-in-root-unchanged", "only-the-file-column-changes", "remap-counts", "remap-exit-codes")),
    ("main-root-undetected", ("root-moved-to-main-detected",)),
    ("new-submodules-undetected", ("split-detected-from-trees", "crate-root-siblings-searched")),
    ("crate-root-siblings-unsearched", ("crate-root-siblings-searched",)),
    ("existing-files-searched", ("crate-root-siblings-searched",)),
    ("fields-reserialized", ("only-the-file-column-changes", "row-order-kept")),
    ("row-order-sorted", ("only-the-file-column-changes", "row-order-kept", "test-columns-only-change")),
    ("unresolved-exit-zero", ("remap-exit-codes", "split-detected-from-trees")),
    ("unchanged-counted-as-changed", ("anchor-staying-in-root-unchanged", "remap-counts")),
    ("registry-subdirectories-scanned", ("registry-fixtures-excluded",)),
    ("registry-path-boundary-ignored", ("registry-path-boundary",)),
    ("registry-non-tsv-scanned", ("registry-only-tsv",)),
    ("sixth-column-not-renamed", ("test-names-whole-replaced", "renamed-test-unselected-listed", "test-columns-only-change", "test-remap-counts", "renames-without-split", "renames-and-file-column-together", "target-renames-by-target", "untargeted-renames-ambiguous-listed", "unrecognized-target-listed")),
    ("exact-filter-not-renamed", ("test-names-whole-replaced", "test-remap-counts", "test-remap-exit-codes", "renames-without-split", "renames-and-file-column-together", "target-renames-by-target", "untargeted-renames-ambiguous-listed", "unrecognized-target-listed")),
    ("renames-by-substring", ("test-names-whole-replaced",)),
    ("substring-filter-renamed", ("substring-filter-lost-listed",)),
    ("lost-filter-not-listed", ("substring-filter-lost-listed", "test-remap-counts")),
    ("kept-filter-listed", ("substring-filter-kept-untouched", "test-remap-counts", "test-remap-exit-codes")),
    ("renamed-test-selection-unchecked", ("renamed-test-unselected-listed", "test-remap-counts")),
    ("unknown-option-ignored", ("unknown-option-listed",)),
    ("columns-reserialized", ("test-names-whole-replaced",)),
    ("listed-tests-exit-zero", ("test-remap-exit-codes", "untargeted-renames-ambiguous-listed", "unrecognized-target-listed")),
    ("renames-alone-refused", ("renames-without-split",)),
    ("renames-union-last-only", ("renames-table-read", "target-renames-table-read")),
    ("conflicting-renames-accepted", ("renames-table-read", "target-renames-table-read")),
    ("malformed-rename-line-skipped", ("renames-table-read",)),
    ("target-ignored", ("target-renames-by-target", "target-renames-registry")),
    ("package-ignored", ("target-renames-by-target", "target-renames-registry")),
    ("test-bin-flags-unparsed", ("target-renames-by-target", "unrecognized-target-listed", "target-renames-registry")),
    ("several-targets-first-taken", ("unrecognized-target-listed",)),
    ("unrecognized-target-renamed", ("unrecognized-target-listed", "target-renames-registry")),
    ("unrecognized-target-not-listed", ("unrecognized-target-listed", "target-renames-registry")),
    ("loose-ambiguity-ignored", ("untargeted-renames-ambiguous-listed",)),
    ("malformed-target-accepted", ("target-renames-table-read",)),
    ("renames-conflict-across-targets", ("target-renames-table-read",)),
    ("registry-test-names-not-scanned", ("registry-test-names", "target-renames-registry")),
    ("registry-test-name-boundary-ignored", ("registry-test-names",)),
    ("trailing-comma-kept", ("rustfmt-normal-form", "rustfmt-only-green", "rustfmt-only-counts")),
    ("single-expression-braces-kept", ("rustfmt-normal-form", "rustfmt-only-green")),
    ("arm-comma-not-added", ("rustfmt-normal-form", "rustfmt-only-green")),
    ("single-element-tuple-comma-stripped", ("rustfmt-normal-form", "rustfmt-real-changes-red")),
    ("statement-braces-dropped", ("rustfmt-normal-form",)),
    ("layout-not-paired", ("rustfmt-only-green", "rustfmt-only-counts")),
    ("layout-counted-as-identical", ("rustfmt-only-green", "rustfmt-only-counts")),
    ("unchanged-files-compared", ("unchanged-files-silent", "new-mod-heads-scaffold")),
    ("mod-head-extra-red", ("unchanged-files-silent", "new-mod-heads-scaffold")),
    ("mod-head-family-by-own-file", ("new-mod-heads-scaffold",)),
    ("module-attributes-head-scaffolded", ("module-attributes-head-added-red",)),
])
BREAKS = tuple(BREAK_CELLS)

# 当前打开的开关：只在 --selftest 里由入口按它读到的开关名单填，别处是空集
broken = set()


def read_text(path):
    """按字节原样读成文本（不换行符、坏字节转义），写回时一个字节不变。"""
    with open(path, encoding="utf-8", errors="surrogateescape", newline="") as handle:
        return handle.read()


class UsageProblem(Exception):
    """参数不对：str 是哪里不对，howto 是下一步怎么办；退出码 2。"""

    def __init__(self, reason, howto):
        super().__init__(reason)
        self.howto = howto
