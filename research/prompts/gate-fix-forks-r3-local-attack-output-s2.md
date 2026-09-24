T4-Q1 The file is an experiment. The supporting fact is the doc comment stating "E156 rerun, second time, first stage: the cost numbers for the four alloc-basis forks, doing only fork 7 (the defer check) and its prerequisite S1 and anchor" which describes comparing multiple arms. This would be refuted if the doc comment described only a single arm or comparison against an external ground truth.

T4-Q2 The file is an experiment. The supporting fact is the doc comment stating "E158, root choice and repair, four forks, pre registration, first stage: the apparatus to be checked into the results store" which describes comparing multiple arms. This would be refuted if the doc comment described only a single arm or comparison against an external ground truth.

T4-Q3 The file is a tool. The supporting fact is the doc comment stating "Host side: take the two device-side logs recorded by QEMU blklogwrites, and compare them field by field, disk by disk, against the recorded stream obtained by replaying the same write path on the host, same parameters, same bytes" which describes comparison against an external ground truth. This would be refuted if the doc comment described multiple arms being compared against each other.

T4-Q4 The file is a tool. The supporting fact is the doc comment stating "this binary only judges what it can judge by itself (recovering the file correctly, and the sequence of on-disk segments), and that a separate question, whether what actually arrived on disk matches what the program believed it sent, is left to row 3's binary to judge afterward on the host side" which indicates it produces data for comparison but does not perform multi-arm comparisons. This would be refuted if the doc comment described comparing multiple arms or experimental results.

T4-Q5 The file is a tool. The supporting fact is the doc comment stating "E142, first-transaction dry run, the implementation side of measurement 5: run scenario:: run_first_transaction on two in-memory disks... and print the bytes written to those 21 regions by the first transaction, one result line per region" and the fact that it outputs data for another binary to compare. This would be refuted if the doc comment described comparing multiple arms or experimental results.

T4-Q6 0. This would be refuted if any row's classification contradicted the filename pattern, such as a tool file with a matching filename or an experiment file without a matching filename.

T4-Q7 A file in research/some-dir/src/bin named my_experiment.rs with doc comment "This experiment compares multiple arms for X" and assertions comparing several arms. This would be refuted if stage 80 included it despite the filename not matching the pattern.

T4-Q8 A file in crates/any-dir/src/bin named e123_tool.rs with doc comment "This tool compares logs against ground truth" and no multi-arm comparisons. This would be refuted if stage 80 excluded it despite the filename matching the pattern.

T8-Q1 Row 1 correct Row 2 correct Row 3 correct Row 4 correct Row 5 correct Row 6 correct Row 7 correct Row 8 correct Row 9 correct Row 10 correct Row 11 correct Row 12 correct Row 13 correct Row 14 correct Row 15 correct Row 16 correct Row 17 correct Row 18 correct Row 19 correct Row 20 correct Row 21 correct Row 22 correct Row 23 correct Row 24 correct

T8-Q2 24 correct 0 half-stripped-word 0 dropped-whole-word-unnecessarily 0 left-a-bare-reference. This would be refuted if any row's output matched a non-correct category.

T8-Q3 Both rows are correct. Step one alone is sufficient to explain why they end up the same. This would be refuted if step four's registered-set check was required to produce the same output.

T8-Q4 HDD4 ends with a digit, matching the tail-id pattern which requires uppercase letters followed by digits, whereas NVMe and SSD do not end with digits. This would be refuted if a word like NVMe4 was not treated as a tail id.

T8-Q5 Row 22 is correct. This would be refuted if the output ended with a bare reference or partial word.

T8-Q6 Step four is responsible because it checks if the tail id is in the registered set. SHA256 is in the set so it remains, while D22 and E142 are not. This would be refuted if step two alone accounted for the difference.

T8-Q7 Step one is responsible. This would be refuted if step four was required to produce the output.
