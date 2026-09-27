SINGLEFS DEFINITION CHANGES - LOCAL ATTACK LEG - ROUND defs-m2-closeout-r2

You are one of several independent reviewers checking a batch of definition
and rule changes in singlefs, a copy on write filesystem being designed from
scratch. This project runs a shell hook, called the heavy test hook, that
inspects a proposed shell command before it is allowed to run, and refuses
certain kinds of commands that this project classifies as heavy tests
(things like running a full QEMU based test, running the herd7 memory model
checker, or running the project's entire compiled test suite at once). When
the heavy test hook refuses a command, the tool call that would have run it
exits with status code 2 instead of running; when the hook allows the
command, the underlying tool call proceeds, and in the specific test records
given to you below, that recorded attempt exits with status code 0.

The classification logic inside that hook lives in two Python functions,
given to you in full below as source code, named classify and cargo_use. One
of this project's rule documents also contains a checklist, written in
prose for human readers, that is supposed to describe, in its own words,
exactly which concrete kinds of commands those two functions classify as
heavy. Separately, an engineer produced a probe log: a list of concrete
example commands, each one fed synthetically to the heavy test hook without
ever actually running the underlying command, together with the exit code
the hook actually produced for it that day.

Your task in this round is a mechanical, three way cross check among the
checklist, the two Python functions, and the probe log. You are given the
full text of the checklist, broken into individually numbered write style
items below (item codes starting with the letter C), the full text of the
probe log, broken into individually numbered log entries below (item codes
starting with the letter K), and the full source code of both Python
functions together with everything else needed to read them, in a section
below called the source material. Do not assume anything about any of the
three beyond what is written below. Read every item carefully; a write
style item that differs from another by only one word, such as a different
flag, a different target, or a different function name, is a different
write style, not the same one.

This round has other independent reviewers looking at different, broader
questions about this same batch of changes, using separate material; you
are not being asked to judge whether the checklist itself is a good idea,
only whether its wording matches what the two given Python functions
actually do, and whether both of those agree with the probe log.

Do not use any markdown emphasis anywhere in your answer: no bold text, no
italic text, no backtick code formatting, no asterisk bullets, no pipe
tables, no heading marks. Answer entirely in English. Do not cite, invent,
or guess at any source file name, file path, or line number anywhere in
your answer, including any you might work out for yourself while
reasoning; refer only to the item codes given below (C codes and K codes),
the group numbers Q1 through Q16 used in the questions below, the plain
function names classify, cargo_use, heavy_test, gate_stage_number, and
test_binary_name, and the plain kind name strings quoted in the source
material below (for example layer0-stage, full-cargo, herd7, or e152).

EXHIBIT A. THE CHECKLIST, BROKEN INTO WRITE STYLE ITEMS

Group C1 covers the checklist's bullet about layer 0 tests being heavy.

C1.1 EXPECTED REJECT. Running gate stage 54 (the layer 0 replay stage
script), with any arguments.
C1.2 EXPECTED REJECT. A cargo test or cargo t invocation that includes a
--test argument whose value either contains the substring layer0, or is a
glob pattern that matches the name of at least one compiled test target
that contains the substring layer0.
C1.3 EXPECTED REJECT. A cargo test or cargo t invocation that does not
narrow its target selection at all (meaning it has none of --test, --lib,
--bin, --bins, --example, --examples, --bench, --benches, or --doc), or
that does narrow using --tests or --all-targets, where the set of packages
actually being tested contains at least one compiled test target whose
name contains the substring layer0.
C1.4 EXPECTED REJECT. Directly executing a compiled test binary file, not
through cargo at all, whose own file name, with its trailing 16 character
hexadecimal hash suffix removed, contains the substring layer0.
C1.5 EXPECTED ALLOW. A cargo test or cargo t invocation that does narrow
its target selection using something other than a layer0 named test, such
as --lib, where no --test argument names or glob matches a target whose
name contains layer0.

Group C2 covers the checklist's bullet about QEMU being heavy.

C2.1 EXPECTED REJECT. Running gate stage 55.
C2.2 EXPECTED REJECT. Running any command whose own file name starts with
the literal text qemu-system.
C2.3 EXPECTED REJECT. Running the script vm-bench.sh, including when it is
given only the single argument --selftest.

Group C3 covers the checklist's bullet about herd7 being heavy.

C3.1 EXPECTED REJECT. Running gate stage 57.
C3.2 EXPECTED REJECT. Running the script lkmm.sh, with any arguments at
all, including an argument that only asks for a version number.
C3.3 EXPECTED REJECT. Running the herd7 command itself, with any arguments
at all, including an argument that only asks for a version number.

Group C4 covers the checklist's bullet about the full crates mutation table
being heavy.

C4.1 EXPECTED REJECT. Running gate stage 59.
C4.2 EXPECTED REJECT. Running the script mutate.sh with an argument that
is, or that resolves to a path ending in, crates/mutations.tsv.

Group C5 covers the checklist's bullet about a full cargo test run being
heavy.

C5.1 EXPECTED REJECT. A cargo test or cargo t invocation given either
--all or --workspace.
C5.2 EXPECTED REJECT. A cargo test or cargo t invocation whose current
working directory has a workspace root manifest with no package section of
its own (either the repository's own root, or the research directory),
given neither -p nor any target narrowing option.
C5.3 EXPECTED REJECT. A cargo test or cargo t invocation given no target
narrowing option, where the set of packages actually being tested is
exactly equal to the complete set of members declared by that command's
own workspace, even when that workspace happens to have exactly one member
and the command was run inside that one member's own directory rather than
at a directory that is itself a workspace root manifest (the checklist
gives running this bare inside the directory research/e7-index-bench/ as
its own explicit worked example).
C5.4 EXPECTED REJECT. Running the script check.sh, with any arguments.
C5.5 EXPECTED ALLOW. A cargo test or cargo t invocation given some target
narrowing option, such as --lib, together with -p naming one specific
package, even when that one named package happens to be the only member of
its workspace; the checklist's own wording for the previous item only
covers the case where no target narrowing option is given at all.

Group C6 covers the checklist's bullet about a whole gate run being heavy.

C6.1 EXPECTED REJECT. Running the script gate.sh, with any arguments
including none.
C6.2 EXPECTED REJECT. Running the script gate-staged.sh with any arguments
other than exactly the single argument --selftest.
C6.3 EXPECTED ALLOW. Running the script gate-staged.sh with exactly the
single argument --selftest.

Group C7 covers the checklist's bullet about replaying every experiment
being heavy.

C7.1 EXPECTED REJECT. Running gate stage 87, with any arguments.

Group C8 covers the checklist's bullet about the E152 apparatus being
heavy.

C8.1 EXPECTED REJECT. Directly executing the e152-file-system-benchmark
binary, or running it through cargo run --bin naming that same binary.
C8.2 EXPECTED REJECT. Running the script e152-run.sh.

Group C9 covers the checklist's paragraph stating that classification looks
only at command position, and that a command otherwise matching one of the
write styles above still counts as heavy even when it is wrapped.

C9.1 EXPECTED REJECT. A command that would otherwise match one of the
write styles in groups C1 through C8, but is itself wrapped inside bash -c,
capped.sh, run-with-memory-cap.sh, nice, timeout, or env, still counts as
that same heavy write style, exactly as if the wrapper were not there.

Group C10 covers the same paragraph's statement that a script run at
command position gets its own file read one line at a time, with each line
judged the same way, and that if any one line matches a heavy write style
the whole invocation counts as that write style.

C10.1 EXPECTED REJECT. A script run at command position, for example
through bash, through a leading ./, or through source, whose own file,
read one line at a time, contains at least one line that would itself be
classified as one of the write styles in groups C1 through C8; the
checklist gives fetch-deps.sh, run with the single argument --check, as a
worked example, stating that one specific line inside that script's own
file calls herd7 asking only for its version, and that the whole
fetch-deps.sh --check invocation is therefore classified the same way that
lone herd7 line would be classified on its own.

Group C11 covers the same paragraph's statement about scripts that are
judged by name alone.

C11.1. The checklist states this as a plain fact about the code, not
phrased as either an expected reject or an expected allow. Gate stage
scripts, and scripts registered by name in a fixed table inside the same
Python file that also holds classify and cargo_use (a table the checklist
calls KNOWN_SCRIPT_LOCATIONS), are judged purely by their own file name and
location, without the hook ever reading those scripts' own file contents
line by line.

Group C12 covers the same paragraph's closing statement about gate stages
that are not heavy at all.

C12.1 EXPECTED ALLOW. Running any gate stage under .claude/gate.d/ other
than stage 54, 55, 57, 59, or 87; the checklist states that anyone may run
those other stages.

END OF EXHIBIT A.

EXHIBIT B. THE PROBE LOG, BROKEN INTO LOG ENTRIES

Background facts about this repository, given to you here because the
source code in exhibit C reads information whose actual content cannot be
seen from that code alone. Treat every sentence in this paragraph as a
given fact, not something to infer.

The repository's own root manifest is a workspace only manifest (it has a
workspace section and no package section of its own), and its workspace
has many member packages, including one named singlefs-harness. The
directory research/ has its own separate manifest, also a workspace only
manifest (a workspace section, no package section), whose workspace has
exactly one member package, named e7-index-bench, located at
research/e7-index-bench/. The directory research/e7-index-bench/ has its
own manifest, which is a package manifest (a package section, no workspace
section of its own) for that one member package. Compiled test binaries
with names containing the substring layer0 exist among the singlefs-harness
package's own test targets; one concrete example name appearing elsewhere
in this material is second_transaction_step_zero_layer0, and at least two
more such names exist whose full text is not given to you here. No
compiled test target anywhere among the e7-index-bench package's own test
targets has a name containing the substring layer0.

Each numbered entry below names the exact concrete example command that
was fed to the heavy test hook, the current working directory it was fed
with, and the exit code the hook actually produced for that exact example
that day.

K1. Command: herd7 -version. Current directory: the repository root. EXIT
2.
K2. Command: qemu-system-x86_64 --version. Current directory: the
repository root. EXIT 2.
K3. Command: directly executing the file
./target/release/deps/second_transaction_step_zero_layer0-0123456789abcdef,
a compiled test binary file, not run through cargo. Current directory: the
repository root. EXIT 2.
K4. Command: cargo test, with no other arguments. Current directory: the
repository root. EXIT 2.
K5. Command: cargo test, with no other arguments. Current directory:
research/. EXIT 2.
K6. Command: cargo test, with no other arguments. Current directory:
research/e7-index-bench/. EXIT 2.
K7. Command: cargo t --workspace. Current directory: the repository root.
EXIT 2.
K8. Command: running the script mutate.sh with three arguments naming the
package singlefs-harness, a source file path, and the path
crates/mutations.tsv. Current directory: the repository root. EXIT 2.
K9. Command: running the script lkmm.sh with the single argument
--herd7-version. Current directory: the repository root. EXIT 2.
K10. Command: running the script gate.sh with the single argument
--selftest. Current directory: the repository root. EXIT 2.
K11. Command: running the script gate-staged.sh with the single argument
--selftest. Current directory: the repository root. EXIT 0.
K12. Command: running the script vm-bench.sh with the single argument
--selftest. Current directory: the repository root. EXIT 2.
K13. Command: cargo run --release --bin e152-file-system-benchmark.
Current directory: research/. EXIT 2.
K14. Command: running the script e152-run.sh, with no arguments. Current
directory: the repository root. EXIT 2.
K15. Command: cargo test -p singlefs-harness --tests. Current directory:
the repository root. EXIT 2.
K16. Command: cargo test -p singlefs-harness --test, with the glob pattern
second_transaction_step_zero_* as that option's value. Current directory:
the repository root. EXIT 2.
K17. Command: running the script fetch-deps.sh with the single argument
--check; one specific line inside this script's own file itself calls
herd7 asking only for its version, exactly like the command in K1 above.
Current directory: the repository root. EXIT 2.
K18. Command: cargo test -p singlefs-harness --lib. Current directory: the
repository root. EXIT 0.
K19. Command: running the script check.sh, with no arguments. Current
directory: the repository root. EXIT 2.
K20. Command: running gate stage 87, with no arguments. Current directory:
the repository root. EXIT 2.
K21. Command: cargo test -p singlefs-harness --test
second_transaction_step_zero_layer0. Current directory: the repository
root. EXIT 2.
K22. Command: running gate stage 54, with no arguments. Current directory:
the repository root. EXIT 2.
K23. Command: running gate stage 57, with no arguments. Current directory:
the repository root. EXIT 2.
K24. Command: running gate stage 59, with no arguments. Current directory:
the repository root. EXIT 2.
K25. Command: running gate stage 55, with no arguments. Current directory:
the repository root. EXIT 2.
K26. Command: cargo test --all. Current directory: the repository root.
EXIT 2.
K27. Command: running the script gate-staged.sh, with no arguments.
Current directory: the repository root. EXIT 2.
K28. Command: cargo test -p singlefs-harness, with no other arguments at
all (no target narrowing option of any kind). Current directory: the
repository root. EXIT 2.
K29. Command: cargo test -p e7-index-bench --lib. Current directory:
research/. EXIT 0.

END OF EXHIBIT B.

EXHIBIT C. THE SOURCE MATERIAL

Below is the full source of the two Python functions that this project's
checklist in exhibit A claims to implement its classification logic
(classify and cargo_use), together with every short helper function,
constant table, and regular expression that these two functions call or
reference directly by name. Two Chinese language documentation comments
that appeared inside the original Python source have been translated into
English below, and a handful of Chinese language message fragments that
get embedded into human readable rejection text have likewise been
translated into English below; nothing about the code's own conditions,
branches, or returned kind name strings has been changed by that
translation.

Six more helper functions are called by cargo_use by name, using
arguments already visible in the code below, but their own bodies are not
shown to you: cargo_subcommand, manifest_sections, nearest_manifest,
workspace_root_manifest, workspace_packages, and layer0_test_targets. A
seventh helper function, resolve_path, is called by name in a couple of
places in the code below, also without its own body shown to you. Reason
about what each of these seven returns only from its own plain English
name, from its arguments, and
from how its return value is used in the code shown to you, together with
the background facts about this repository already given to you above
exhibit B. Every concrete command in exhibit B that begins with the word
cargo has an ordinary subcommand word appearing immediately after cargo
with no other option word in between, so you may assume cargo_subcommand
simply returns that subcommand word itself, every argument after it
unchanged, and the original current directory unchanged, for every such
command in exhibit B.

SOURCE BEGINS.

GATE_STAGE_PATH = a regular expression matching a command word that either
is, or ends with a path separator followed by, the literal text
.claude/gate.d/ followed by one or more digits, followed by a hyphen, one
or more characters that are not a path separator, and the literal text
.sh, at the very end of the word; the digits are captured as group 1.

TEST_BINARY_PATH = a regular expression matching a command word that
either is, or ends with a path separator followed by, the literal text
deps/ followed by one or more letters, digits, or underscores, followed by
a hyphen, followed by exactly 16 lowercase hexadecimal digits, at the very
end of the word; the letters, digits, or underscores right after deps/ are
captured as group 1.

NARROWING_SELECTORS = the set containing exactly these option strings:
--test, --lib, --bin, --bins, --example, --examples, --bench, --benches,
--doc.

GLOB_CHARACTERS = the set containing exactly these three characters: the
asterisk, the question mark, and the open square bracket.

CARGO_TEST_OPTIONS_WITH_VALUE = a set of option strings that each take a
following value argument, including among others -p, --package, --test,
--bin, --example, --bench, and --manifest-path.

STAGE_KIND = a table mapping the stage number string "54" to the kind name
layer0-stage, "55" to qemu-stage, "57" to herd7-stage, "59" to
crates-mutation-stage, and "87" to replay-all-stage. Comment on this table
in the original source, translated: only these stage numbers are heavy;
every other stage under .claude/gate.d/ may be run by anyone.

KIND_CATEGORY = a table mapping each of these kind name strings to a
human readable category name: layer0-stage, layer0-cargo, and
layer0-binary all map to the category named layer 0. qemu-stage,
qemu-system, and vm-bench all map to the category named QEMU. herd7-stage,
lkmm, and herd7 all map to the category named herd7. crates-mutation-stage
and crates-mutation-mutate both map to the category named crates mutation
table. full-cargo and check-sh both map to the category named full test
suite. gate-sh and gate-staged both map to the category named whole gate
run. replay-all-stage maps to the category named replay every experiment.
e152 maps to the category named E152 apparatus.

def heavy_test(kind, detail):
    return a record combining kind, KIND_CATEGORY[kind], and detail.

def gate_stage_number(word, directory):
    for candidate in (word, resolve_path(directory, word) or ""):
        match = GATE_STAGE_PATH.search(candidate)
        if match:
            return match.group(1)
    return None

def test_binary_name(word):
    # Translated documentation comment: if the word being directly executed
    # is a cargo built test binary, return its name with the hash suffix
    # removed; otherwise return None.
    match = TEST_BINARY_PATH.search(word)
    return match.group(1) if match else None

def classify(words, directory):
    # Translated documentation comment: a single command whose prefix and
    # wrapper have already been stripped away; returns a heavy_test record
    # or None.
    name, arguments = os.path.basename(words[0]), words[1:]
    stage = gate_stage_number(words[0], directory)
    if stage is not None:
        return heavy_test(STAGE_KIND[stage], f"gate stage {stage} ({name})") if stage in STAGE_KIND else None
    binary = test_binary_name(words[0])
    if binary is not None:
        return heavy_test("layer0-binary", f"directly executing a test binary whose name contains layer0 ({binary})") if "layer0" in binary else None
    if name.startswith("qemu-system"):
        return heavy_test("qemu-system", name)
    if name == "vm-bench.sh":
        return heavy_test("vm-bench", "research/scripts/vm-bench.sh (--selftest also starts a virtual machine)")
    if name == "lkmm.sh":
        return heavy_test("lkmm", ".claude/scripts/lkmm.sh")
    if name == "herd7":
        return heavy_test("herd7", "herd7")
    if name == "mutate.sh":
        for argument in arguments:
            resolved = resolve_path(directory, argument) or argument
            if argument.endswith("crates/mutations.tsv") or resolved.endswith("/crates/mutations.tsv"):
                return heavy_test("crates-mutation-mutate", "research/scripts/mutate.sh running the whole crates/mutations.tsv table")
        return None
    if name == "check.sh":
        return heavy_test("check-sh", "check.sh (which internally runs the full cargo test suite)")
    if name == "gate.sh":
        return heavy_test("gate-sh", " ".join(["gate.sh", *arguments]))
    if name == "gate-staged.sh":
        return None if "--selftest" in arguments else heavy_test("gate-staged", "research/scripts/gate-staged.sh (running gate.sh --staged)")
    if name in ("e152-file-system-benchmark", "e152-run.sh"):
        return heavy_test("e152", name)
    if name == "cargo":
        return cargo_use(arguments, directory)
    return None

def cargo_use(arguments, directory):
    # Translated documentation comment: whether this cargo invocation
    # counts as heavy; returns a heavy_test record or None.
    found = cargo_subcommand(arguments, directory)
    if found is None:
        return None
    subcommand, rest, directory = found
    packages, test_names, selectors, binaries = [], [], set(), []
    whole_workspace, manifest_argument = False, None
    position = 0
    while position < len(rest):
        argument = rest[position]
        if argument == "--":
            break
        if argument.startswith("--") and "=" in argument:
            option, value = argument.split("=", 1)
            position += 1
        elif argument.startswith("-p") and len(argument) > 2 and not argument.startswith("--"):
            option, value = "-p", argument[2:].lstrip("=")
            position += 1
        elif argument in CARGO_TEST_OPTIONS_WITH_VALUE:
            option = argument
            value = rest[position + 1] if position + 1 < len(rest) else ""
            position += 2
        else:
            option, value = argument, None
            position += 1
        if option in ("-p", "--package"):
            packages.append(value)
        elif option == "--test":
            test_names.append(value)
            selectors.add(option)
        elif option in ("--bin", "--example", "--bench"):
            selectors.add(option)
            if option == "--bin":
                binaries.append(value)
        elif option == "--manifest-path":
            manifest_argument = value
        elif option in ("--workspace", "--all"):
            whole_workspace = True
        elif option in ("--lib", "--bins", "--examples", "--tests", "--benches", "--all-targets", "--doc"):
            selectors.add(option)
    if subcommand == "run":
        if "e152-file-system-benchmark" in binaries:
            return heavy_test("e152", "cargo run --bin e152-file-system-benchmark")
        return None
    if subcommand not in ("test", "t"):
        return None
    if whole_workspace:
        return heavy_test("full-cargo", "cargo test given --workspace or --all")
    named_layer0 = [name for name in test_names if "layer0" in name]
    if named_layer0:
        return heavy_test("layer0-cargo", f"cargo test --test {named_layer0[0]}")
    manifest_path = resolve_path(directory, manifest_argument) if manifest_argument else (nearest_manifest(directory) if directory else None)
    sections = manifest_sections(manifest_path) if manifest_path else None
    if sections is None:
        return None
    is_virtual_workspace_root = "workspace" in sections and "package" not in sections
    narrowed = bool(selectors & NARROWING_SELECTORS)
    if not packages and is_virtual_workspace_root and not narrowed:
        return heavy_test("full-cargo", f"cargo test, given neither -p nor --test/--lib/--bin, at a workspace root ({os.path.dirname(manifest_path)})")
    root_manifest, root_sections = workspace_root_manifest(manifest_path)
    members = workspace_packages(root_manifest, root_sections) if root_manifest else {}
    if packages:
        scope = [members[name] for name in packages if name in members]
    elif is_virtual_workspace_root:
        scope = list(members.values())
    else:
        scope = [os.path.normpath(os.path.dirname(manifest_path))]
    if not narrowed and members and set(scope) == set(members.values()):
        return heavy_test("full-cargo", (f"cargo test with no target narrowing, where the package scope is the workspace's entire membership "
                                         f"({os.path.dirname(root_manifest)}'s full {len(members)} members), which counts as the whole workspace"))
    layer0_targets = sorted({name for package_directory in scope for name in layer0_test_targets(package_directory)})
    if not layer0_targets:
        return None
    if not selectors or selectors & {"--tests", "--all-targets"}:
        return heavy_test("layer0-cargo", f"cargo test with no target narrowing, which will run compiled test binaries whose names contain layer0 ({', '.join(layer0_targets)})")
    for pattern in test_names:
        if GLOB_CHARACTERS & set(pattern):
            matched = fnmatch.filter(layer0_targets, pattern)
            if matched:
                return heavy_test("layer0-cargo", f"cargo test --test {pattern}, which matches compiled test binaries whose names contain layer0 ({', '.join(matched)})")
    return None

SOURCE ENDS. END OF EXHIBIT C.

HOW TO ANSWER Q1 THROUGH Q12

For every C item inside one of these questions' groups, decide the
following three things and report them together on one line, in this fixed
order: MATCH, meaning the code or codes of every K item, if any, whose
exact concrete example is a genuine instance of that exact C item's write
style (or the single word NONE if no K item is a genuine instance of it);
EXITCODE, meaning the exit code or codes actually recorded on the K item or
items you just named (or the words NOT APPLICABLE if you named NONE); and
VERDICT, which must be exactly one of the three words CONFIRMED,
CONTRADICTED, or UNTESTED. VERDICT is CONFIRMED when at least one matching
K item exists and every matching K item's exit code agrees with that C
item's own EXPECTED label (2 for EXPECTED REJECT, 0 for EXPECTED ALLOW; for
C11.1, which carries neither label, treat it as EXPECTED ALLOW, since the
checklist describes it as a script whose contents are never even read
rather than as something refused). VERDICT is CONTRADICTED when at least
one matching K item exists and at least one matching K item's exit code
disagrees with that expectation. VERDICT is UNTESTED when you named NONE.
Judge a match strictly: a K item matches a C item only when the K item's
own concrete example is a real instance of the exact write style that C
item describes, not merely something in the same general family (a
different flag, a different tool, or a different target counts as not
matching). Never answer a C item with only the word yes, only the word no,
or by leaving any of the three parts blank; always give the specific codes
and the specific words defined above.

QUESTIONS Q1 THROUGH Q12

Q1. One answer line for each of C1.1 through C1.5.
Q2. One answer line for each of C2.1 through C2.3.
Q3. One answer line for each of C3.1 through C3.3.
Q4. One answer line for each of C4.1 and C4.2.
Q5. One answer line for each of C5.1 through C5.5.
Q6. One answer line for each of C6.1 through C6.3.
Q7. One answer line for C7.1.
Q8. One answer line for each of C8.1 and C8.2.
Q9. One answer line for C9.1.
Q10. One answer line for C10.1.
Q11. One answer line for C11.1, using the same MATCH, EXITCODE, VERDICT
format, treating C11.1 as EXPECTED ALLOW as instructed above.
Q12. One answer line for C12.1.

Q13. Read back over every answer line you gave for Q1 through Q12. List
the code of every single C item whose VERDICT you gave was CONTRADICTED.
If there are none, say the single word NONE. For every code you list here,
also restate, in one sentence per code, the specific K item and its exit
code that caused you to call it CONTRADICTED.

Q14. Now work independently of your own MATCH answers above. Go through
every single K item in exhibit B whose EXIT is 2, one at a time. For each
one, decide for yourself, directly from the wording of exhibit A, whether
any single C item names that exact write style exactly, not
approximately. List the code of every K item with EXIT 2 for which no C
item names that exact write style exactly, even if you used that K code as
a MATCH somewhere in your answers to Q1 through Q12; if you find one you
had used as a MATCH, say so explicitly and explain in one sentence why you
now think that naming was only approximate rather than exact. If every
single K item with EXIT 2 is named exactly by some C item, say the single
word NONE.

Q15. Work only from exhibit C, the source material, not from exhibit B's
probe results. For each of the 28 individual C items across groups C1
through C12, decide whether the classify or cargo_use source code actually
contains a branch or condition implementing that exact write style. If it
does, name it using only the plain kind name string that branch returns
(for example layer0-stage, full-cargo, herd7), or, if that branch returns
no kind name string of its own because the write style is entirely handled
by a black box helper not shown to you, name that helper function instead
and say so explicitly. If no branch or condition anywhere in the given
source implements that exact write style, answer with the exact words NOT
IN SOURCE and explain in one sentence which specific behavior described by
that checklist item is missing from classify and cargo_use as given to
you. Give one answer line per C item, in the same order as Q1 through Q12,
each line still labelled with that item's own C code.

Q16. Now work only from exhibit C, independently of every answer you gave
above. List every distinct kind name string that appears anywhere in the
KIND_CATEGORY table. For each one, decide whether any single C item in
exhibit A names that exact write style. If yes, give that C item's code.
If no single C item names it, answer with the exact words NOT IN CHECKLIST
and describe, in one plain sentence, without using any file name or line
number, the concrete circumstance under which classify or cargo_use
returns that kind, using only what is shown to you in exhibit C.

FORMAT RULES

Give exactly sixteen numbered answers, labeled Q1 through Q16, in that
exact order. For Q1 through Q12, give one answer line per C item in that
question's range, using the MATCH, EXITCODE, VERDICT format described
above, with every C item's own code repeated at the start of its line. For
Q13 through Q16, answer in the format each of them separately describes
above. After each individual C item or K item answer you give anywhere in
Q1 through Q16, on its own line, add one sentence starting with exactly
the words This would be refuted by: followed either by one specific K code
and the exact different exit code it would have to show in exhibit B, or
by one specific, concrete change to the wording of exhibit A or exhibit C,
for that same answer to become wrong. Do not use any markdown emphasis
anywhere in your answer: no bold text, no italic text, no backtick code
formatting, no asterisk bullets, no pipe tables, no heading marks. Do not
cite, invent, or guess at any source file name, file path, or line number
anywhere in your answer; refer only to C codes, K codes, question numbers
Q1 through Q16, plain function names, and plain kind name strings, all as
given above. Answer entirely in English.

END OF MATERIAL. Answer Q1 through Q16 now, in the format given above.
