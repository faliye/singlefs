**Part One**  
1. Label A: The phrase "not verified in this project" references the consumer's specific project, violating Rule 8's prohibition against naming the project that uses the SOP. This would be wrong if the SOP used a generic term like "the current project" instead of "this project".  
2. Label A: "the project's own kb/" explicitly names a consumer-specific path, which Rule 8 forbids in the SOP body text. This would be wrong if the SOP referred to a generic path like ".claude/rules/" without "project's own".  
3. Label B: "singlefs-ai-sop" is the SOP repository's own name (not the downstream project's name), so it complies with Rule 8. This would be wrong if "singlefs-ai-sop" referred to the consumer's project name.  
4. Label B: "logical block address: the number the filesystem gives a block" is a generic definition with no project-specific references. This would be wrong if it specified a project's unique filesystem implementation.  
5. Label B: ".claude/rules/" is a generic path for any consumer's project-specific rules, consistent with Rule 8. This would be wrong if it named a specific project's directory like "singlefs/.claude/rules/".  
6. Label B: "It has exactly one user" is a general statement about the SOP's scope, not naming a specific project. This would be wrong if the SOP listed a specific consumer name in "consumers=" elsewhere.  
7. Label B: "the project" refers generically to any consumer's project, aligning with Rule 8's allowance for generic terms. This would be wrong if it named a specific project like "singlefs's design decisions".  
8. Label B: "consumers=singlefs" in I18N is explicitly permitted by Rule 8 as the sole consumer list location. This would be wrong if the I18N file contained multiple consumers or the SOP body text duplicated it.  
9. Label A: "matching singlefs" names the downstream project in the License section, violating Rule 8's ban on consumer-specific references. This would be wrong if the Chinese edition omitted "singlefs" and the URL.  
10. Label A: "E152" is an experiment number used as an example, which Rule 8 prohibits in the SOP body text. This would be wrong if E152 was a generic term like "a performance test".  
11. Label A: "E152's run_singlefs" combines an experiment number and a project-specific command name, violating Rule 8. This would be wrong if it referenced a generic example like "a test for timestamping".  
12. Label B: The guideline about "table" references is a generic rule for any filesystem project, not project-specific. This would be wrong if it cited a specific project's documentation practices.  
13. Label B: "ext: extent / extension / external" is a generic term expansion for filesystem projects, compliant with Rule 8. This would be wrong if it tied the expansion to a specific project's usage.  
14. Label B: The Japanese excluded list is a generic system-under-test term list, not project-specific. This would be wrong if it included a project-specific word like "singlefs-disk".  

**Part Two**  
| Command | Category | Rule Block Reference |  
|---------|----------|----------------------|  
| 1 | harness | Rule Block One's "harness: crates/singlefs-harness" section |  
| 2 | harness | Rule Block One's "harness: crates/singlefs-harness" section |  
| 3 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 4 | layer 0 | Rule Block Two's "heavy testing is that whole checker tier: tests in the singlefs-checker package, layer 0, crash-enumeration cases..." |  
| 5 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 6 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 7 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 8 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 9 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 10 | harness | Rule Block One's "harness: crates/singlefs-harness" section |  
| 11 | full test suite | Rule Block Two's "heavy testing is that whole checker tier... full cargo test..." |  
| 12 | full test suite | Rule Block Two's "heavy testing is that whole checker tier... full cargo test..." |  
| 13 | full test suite | Rule Block Two's "heavy testing is that whole checker tier... full cargo test..." |  
| 14 | checker tier | Rule Block One's "checker tier test for membership: a test that enumerates crash states..." |  
| 15 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 16 | checker tier | Rule Block One's "checker tier test for membership: a test that enumerates crash states..." |  
| 17 | layer 0 | Rule Block Two's "heavy testing is that whole checker tier: tests in the singlefs-checker package, layer 0..." |  
| 18 | harness | Rule Block One's "harness: crates/singlefs-harness" section |  
| 19 | layer 0 | Rule Block Two's "heavy testing is that whole checker tier... QEMU (gate stage 55), herd7 (stage 57), layer 0..." |  
| 20 | not heavy | Rule Block Two's "Every other stage under the project's gate-stage directory is not heavy, and any role may run it" |  
| 21 | not heavy | Rule Block Two's "heavy testing is that whole checker tier... [excluding excludes listing tests]" |  
| 22 | not heavy | Rule Block Two's "any role may run it" for non-heavy stages (singlefs-format not in heavy categories) |  
| 23 | harness | Rule Block One's "runs any time the code changes... plus fmt / clippy / build" |  
| 24 | crates mutation table | Rule Block Two's "heavy testing is that whole checker tier... the full crates mutation table..." |  
| 25 | checker tier | Rule Block One's "checker: crates/singlefs-checker" section |  
| 26 | crates mutation table | Rule Block Two's "heavy testing is that whole checker tier... the full crates mutation table (stage 59)" |  

This would be wrong if any command's classification contradicted the explicit wording in Rule Block One or Rule Block Two, such as a "cargo test -p singlefs-checker" command being classified as not heavy or a gate stage 74 command being labeled as heavy.
