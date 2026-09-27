TASK

You are checking a small piece of a build tool called admission.py against its own written specification. The tool computes, for each of four registered crash-enumeration test cases in a Rust workspace, the exact set of input files that count toward that case's cache fingerprint. The specification is quoted below as rules R1 through R4. Below that are three fact tables, G1, G2, and G3, each giving numbers or statuses that were produced by actually running the tool's commands against the real repository on 2026-09-26 (not invented, not simulated). Your job is to judge, row by row and column by column, whether the recorded number or status in each table cell is what rules R1 through R4 predict, and, where it is not obviously predicted, to say so and say what would need to be true for it to match. Do not evaluate whether R1 through R4 are good rules; only check whether the recorded facts are consistent with them.

THE SPECIFICATION (RULES)

R1. A crash case's input set equals the files that git lists under its registered paths, which for every one of the four cases here are: crates/ in full, plus the top-level Cargo.toml, plus the top-level Cargo.lock, minus the test files exclusive to other test targets (defined by R2 and R3), plus gate 54's own file, the toolchain description, the build environment description, and this case's own registration row.

R2. A test target named X has files exclusive to it only when all of the following hold about the Rust package that contains X: that package has no build.rs file, no [[test]] entry in its Cargo.toml, no package.autotests key, and no package.build key. Given that, X is defined as either the single file tests/X.rs, or, for a directory-style target, tests/X/main.rs together with every other file under that same tests/X/ directory.

R3. Even when R2 holds for some test target X, X's files are not excluded, and stay in the input set, if X's name occurs as a whole word, after all comments have been stripped out of the surrounding code, in any other .rs file or in any Cargo.toml file anywhere in the workspace. A mod X; statement, a #[path = "...X.rs"] attribute, an include_str!("...X.rs") call, and code that reads X by a literal path string at runtime, are each, on their own, an occurrence of this kind; any single one of them is enough to keep X's files in the input set. Occurrences inside a stripped comment do not count.

R4. The registration for every one of the four cases only ever lists crates/ as a whole; it does not hand-list, case by case, which specific files that case reads. A newly added file that several test targets share, for example tests/common.rs, a newly added build.rs, or a newly split-out crate, therefore defaults to being included in that case's input set, because R1 only ever subtracts files by the R2 and R3 test, it never needs a file to be added to the set by name.

THE FOUR REGISTERED CASES

C1 is crash-case:layer0-first-stream, whose own test target is tests/first_transaction_step_seven_layer0.rs.
C2 is crash-case:layer0-second-stream, whose own test target is tests/second_transaction_step_zero_layer0.rs.
C3 is crash-case:floor-raise-pushed-by-the-session, whose own test target is tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs.
C4 is crash-case:c561-sigma-full, whose own test target is tests/record_checker_judges_absence_by_the_persisted_set.rs.
All four test targets belong to the same Rust package, singlefs-harness, and that package has no build.rs, no [[test]] entry, no package.autotests key, and no package.build key.

FACT TABLE G1, KEPT AND EXCLUDED FILE COUNTS

Each row is the literal stdout of running the command admission.py crash-case-manifest . <case-key> <output-file>, once per case, on 2026-09-26. The three printed numbers are, in order, the fingerprint hash, the count of files kept in the input set, and the count of files excluded from it.

G1 row C1: 8c2cb04e80e3453310762b9d7b7433381032a40d7e11c261fbdafdae74f5ae02 69 68
G1 row C2: e665d2e37e56ef7c2496aa915914e09268441352918263f52919f7fd700d51e0 69 68
G1 row C3: a83e75f86dddbc85ce38451476f42d909bd1fb91f17a7c862463abe899eb25bc 69 68
G1 row C4: b094e0446aa2c6188d7c1855f1267d0c2b7a05deb5376d399df2e89b45937654 69 68

FACT TABLE G2, FOUR CANDIDATE NAMES AND WHETHER THEIR FILES ARE KEPT OR EXCLUDED

Every row's kept-or-excluded status was checked by searching the actual manifest file written by the same crash-case-manifest command as G1, once for each of the four cases; the status shown is identical in all four cases' manifest files for every one of these four names.

G2 row 1, name tests/common/mod.rs: status kept in all four cases' manifest files. This name is never a candidate for exclusion under R2 at all, because the package's tests/ directory has no file named tests/common.rs and no file tests/common/main.rs; only tests/common/mod.rs exists.

G2 row 2, name tests/first_transaction_region_bytes.rs: status kept in all four cases' manifest files. Outside its own file, the word first_transaction_region_bytes occurs four times in the workspace: inside a // line comment in one file, inside a //! doc comment at the top of another file, inside a //! doc comment at the top of a third file, and inside a string literal passed to an eprintln! call in a fourth file (that fourth file is a src/bin program, not the test file itself).

G2 row 3, name tests/instance_acquisition.rs: status kept in all four cases' manifest files. Outside its own file, the word instance_acquisition occurs, among other places, inside four separate string literals, in three different non-test source files (one is an enum-to-string match arm, one is an array of &str, one is a comma-joined status string).

G2 row 4, name tests/checker_known_bad_images.rs: status excluded from all four cases' manifest files. Outside its own file, the word checker_known_bad_images occurs six times in the workspace, in six different files; every one of those six occurrences is inside a // line comment or inside a /// doc comment. None of the six is inside a string literal, a mod statement, a #[path] attribute, or an include_str! call.

FACT TABLE G3, EACH CASE'S OWN TARGET FILE ACROSS ALL FOUR CASES

Each cell's status was obtained the same way as G2, by searching each case's own manifest file for the given file's path.

G3 row 1, file tests/first_transaction_step_seven_layer0.rs (C1's own target): kept in C1's manifest, excluded in C2's manifest, excluded in C3's manifest, excluded in C4's manifest.
G3 row 2, file tests/second_transaction_step_zero_layer0.rs (C2's own target): excluded in C1's manifest, kept in C2's manifest, excluded in C3's manifest, excluded in C4's manifest.
G3 row 3, file tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs (C3's own target): excluded in C1's manifest, excluded in C2's manifest, kept in C3's manifest, excluded in C4's manifest.
G3 row 4, file tests/record_checker_judges_absence_by_the_persisted_set.rs (C4's own target): excluded in C1's manifest, excluded in C2's manifest, excluded in C3's manifest, kept in C4's manifest.

ANSWER FORMAT

Produce exactly 12 numbered answers, labeled G1.1, G1.2, G1.3, G1.4, G2.1, G2.2, G2.3, G2.4, G3.1, G3.2, G3.3, G3.4, in that order.

For every G1 answer: state the kept count and the excluded count you read for that row. State whether that pair of numbers is consistent with rules R1 through R4, given everything else stated in this prompt about the four cases (answer yes or no). If it is not obviously forced by the rules alone, name what additional fact about the repository would have to be true to make it consistent. End with one sentence that starts with the words "this would be falsified by".

For every G2 answer: name which one of R2, R3, or R4 is the rule that determines the recorded status for that row's name. State whether the recorded status, kept or excluded, is what that rule predicts given the occurrences described for that name in this prompt (answer yes or no). If it is not what the rule predicts, say exactly which described occurrence you think is being misjudged, and why. End with one sentence that starts with the words "this would be falsified by".

For every G3 answer: state how many of the four columns show kept for that row's file, as a number from 0 to 4. State whether that number is what rules R1 through R3 predict for a case's own registered test target (answer yes or no). Name which specific column, C1, C2, C3, or C4, is the odd one out, if any one is. End with one sentence that starts with the words "this would be falsified by".

Do not cite source-code line numbers or file line numbers anywhere in your answer. Refer to files by the names given above, to rules by their R-number (R1, R2, R3, R4), and to table rows by their G-number and row number (G1 row 1, G2 row 3, and so on).

Do not use markdown bold, italic, or heading markup anywhere in your reply. Plain numbered lines only.

Do not write prose beyond what is needed to answer the twelve items above. Do not summarize or repeat the facts back to me.

END OF FACTS. Answer G1.1 through G3.4 now, in the format given above.
