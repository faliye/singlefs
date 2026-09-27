G1.1 69 68 yes the number of excluded files must be identical across all cases this would be falsified by any case having a different kept or excluded count  
G1.2 69 68 yes the number of excluded files must be identical across all cases this would be falsified by any case having a different kept or excluded count  
G1.3 69 68 yes the number of excluded files must be identical across all cases this would be falsified by any case having a different kept or excluded count  
G1.4 69 68 yes the number of excluded files must be identical across all cases this would be falsified by any case having a different kept or excluded count  
G2.1 R1 no the file is not part of the registered paths (crates/ Cargo.toml Cargo.lock) so should be excluded this would be falsified by the file being excluded from all manifests  
G2.2 R3 yes the string literal in an eprintln! call counts as an occurrence this would be falsified by the string literal occurrence being removed  
G2.3 R3 yes the string literals in non-test files count as occurrences this would be falsified by all string literal occurrences being removed  
G2.4 R2 yes all occurrences are in comments so exclusion is correct this would be falsified by any non-comment occurrence of the name  
G3.1 1 yes C1 this would be falsified by the file being excluded in C1's manifest or kept in another case  
G3.2 1 yes C2 this would be falsified by the file being excluded in C2's manifest or kept in another case  
G3.3 1 yes C3 this would be falsified by the file being excluded in C3's manifest or kept in another case  
G3.4 1 yes C4 this would be falsified by the file being excluded in C4's manifest or kept in another case
