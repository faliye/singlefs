T4-Q1 experiment This would be refuted if the doc comment described comparing a single ground truth rather than multiple arms

T4-Q2 experiment This would be refuted if the doc comment described comparing a single ground truth rather than multiple arms

T4-Q3 tool This would be refuted if the doc comment described running multiple experimental arms to compare

T4-Q4 tool This would be refuted if the doc comment described comparing multiple arms rather than internal recovery checks

T4-Q5 tool This would be refuted if the doc comment described running multiple experimental arms to compare

T4-Q6 0 This would be refuted if any row had filename matching pattern but classified as tool or filename not matching but classified as experiment

T4-Q7 a file in research/some-dir/src/bin/my_experiment.rs with filename starting with my_experiment and doc comment describing comparison of multiple arms This would be refuted if stage 80 included this file in its scope check

T4-Q8 a file named e123_tool.rs in crates/some-dir/src/bin with doc comment describing comparison to ground truth This would be refuted if stage 80 did not include this file in its scope check

T8-Q1 row 1 correct row 2 correct row 3 correct row 4 correct row 5 correct row 6 correct row 7 correct row 8 correct row 9 correct row 10 correct row 11 correct row 12 correct row 13 correct row 14 correct row 15 correct row 16 correct row 17 correct row 18 correct row 19 correct row 20 correct row 21 correct row 22 correct row 23 correct row 24 correct

T8-Q2 correct 24 half stripped word 0 dropped whole word unnecessarily 0 left a bare reference 0

T8-Q3 both correct category step one alone is sufficient This would be refuted if step four changed the outcome for either row

T8-Q4 HDD4 ends with digits matching the tail id pattern while NVMe and SSD do not This would be refuted if HDD4 was kept in output or NVMe was removed

T8-Q5 correct This would be refuted if HDD4 was treated as a decision id requiring removal

T8-Q6 step four is responsible because SHA256 is in the registered set so it is not removed after step two This would be refuted if SHA256 was removed despite being in the registered set

T8-Q7 step one This would be refuted if step four altered the output for row 19
