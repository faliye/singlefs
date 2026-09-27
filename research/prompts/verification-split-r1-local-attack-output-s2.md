Part One:  
1. B. The phrase "this project" is generic and refers to any project using the SOP, not a specific downstream project like "singlefs". This would be wrong if "this project" explicitly named "singlefs" or another specific project.  
2. B. "Project's own" is a generic term describing any consumer's structure, not tied to a specific project. This would be wrong if "project's own" was replaced with "singlefs's own" or another concrete name.  
3. B. "singlefs-ai-sop" refers to the SOP repository's own name (not the downstream project "singlefs"), which is part of the specification's structure and allowed by Rule 8. This would be wrong if "singlefs-ai-sop" was the name of a downstream project being consumed.  
4. B. "lba" is a standard filesystem term defined generically, with no project-specific references. This would be wrong if "lba" was defined only for singlefs-specific implementations.  
5. B. ".claude/rules/" is a shared path convention (as per Rule 8's "hookup-path shape" allowance), not a concrete file name. This would be wrong if the path included project-specific names like ".claude/rules/singlefs_rules".  
6. B. "exactly one user" is a generic statement about the SOP's design, referencing the I18N file's consumer list without naming a specific project. This would be wrong if the I18N file listed multiple consumers, contradicting "exactly one user".  
7. B. "the project" is a generic term for any consumer, describing where filesystem-specific rules belong. This would be wrong if "the project" explicitly named "singlefs" or another specific project.  
8. B. The I18N file is explicitly permitted by Rule 8 to contain the consumer list under "consumers=", so listing "singlefs" there is compliant. This would be wrong if Rule 8 prohibited consumer lists in the I18N file.  
9. A. "matching singlefs (https://github.com/faliye/singlefs)" names the downstream project directly in the SOP's body text, violating Rule 8. This would be wrong if the Chinese edition's license section omitted "singlefs" like the English/Japanese editions.  
10. A. "E152" is a specific experiment number used as an example, which Rule 8 prohibits in the SOP's body text. This would be wrong if "E152" was replaced with a generic term like "a benchmark experiment".  
11. A. "E152's run_singlefs" references a specific experiment number and a project-specific function name ("run_singlefs"), violating Rule 8. This would be wrong if the example used a generic name like "a test function" instead of "run_singlefs".  
12. B. The rule describes generic filesystem documentation conventions (e.g., "table" usage), not project-specific terminology. This would be wrong if "filesystem" in the excluded list was specific to singlefs's internal terms.  
13. B. "ext" expansions are standard across filesystem projects, with no project-specific references. This would be wrong if the expansions were unique to singlefs (e.g., "ext: singlefs_extent").  
14. B. The Japanese-specific exclusion list adjusts for language conventions, not project-specific naming. This would be wrong if the excluded words included singlefs-specific terms like "singlefs_disk".  

Part Two:  
| Command | Category | Rule Reference |  
|---------|----------|----------------|  
| 1 | harness | Rule Block One's harness section: "cargo test -p singlefs-harness --test <target>" |  
| 2 | harness | Rule Block One's harness section: "cargo test -p singlefs-harness --test <target>" |  
| 3 | checker tier | Rule Block One's "checker: crates/singlefs-checker... its tests/ directory is the checker tier" |  
| 4 | layer 0 | Rule Block Two's POLICY: "heavy testing is that whole checker tier: ... layer 0, ..." |  
| 5 | checker tier | Rule Block One's "checker: crates/singlefs-checker... its tests/ directory is the checker tier" |  
| 6 | checker tier | Rule Block Two: "test binaries in non-checker packages such as singlefs-harness..." (singlefs-checker is checker tier) |  
| 7 | checker tier | Rule Block One's "checker: crates/singlefs-checker... its tests/ directory is the checker tier" |  
| 8 | checker tier | Rule Block One's "checker: crates/singlefs-checker... its tests/ directory is the checker tier" |  
| 9 | checker tier | Rule Block One's "checker: crates/singlefs-checker... its tests/ directory is the checker tier" |  
| 10 | harness | Rule Block One's harness section: "cargo test -p singlefs-harness --test <target>" |  
| 11 | full test suite | Rule Block Two: "heavy testing is ... full cargo test" |  
| 12 | full test suite | Rule Block Two: "heavy testing is ... full cargo test" |  
| 13 | full test suite | Rule Block Two: "heavy testing is ... full cargo test" |  
| 14 | checker tier | Rule Block One: "checker tier — test cases that directly call the full crash-enumeration functions" |  
| 15 | checker tier | Rule Block Two: "heavy testing is ... tests in the singlefs-checker package" |  
| 16 | checker tier | Rule Block One: "full-enumeration case is marked #[ignore]" |  
| 17 | layer 0 | Rule Block Two: "heavy testing is ... layer 0, ..." |  
| 18 | harness | Rule Block One: harness tests (e.g., "second_transaction_step_one_overwrite" is harness-specific) |  
| 19 | layer 0 | Rule Block Two: "crash-verifier role only runs gate stages 54, 55, 57, 59" |  
| 20 | not heavy | Rule Block Two: "Every other stage under the project's gate-stage directory is not heavy" |  
| 21 | not heavy | Rule Block Two: heavy testing involves running tests; "--list" only lists tests without execution |  
| 22 | harness | Rule Block Two: "test binaries in non-checker packages such as singlefs-harness" (singlefs-format is non-checker) |  
| 23 | harness | Rule Block One: "plus fmt / clippy / build" |  
| 24 | crates mutation table | Rule Block Two: "full crates mutation table (stage 59)" |  
| 25 | checker tier | Rule Block Two: "heavy testing is ... tests in the singlefs-checker package" |  
| 26 | crates mutation table | Rule Block Two: "full crates mutation table (stage 59)" |  
This would be wrong if any command's classification contradicted the explicit wording in Rule Block One or Rule Block Two (e.g., if gate stage 74 was classified as heavy when Rule Block Two explicitly states it is not heavy).
