1
P1: specific device offset. Justification: Unit write targets a specific offset as block placement is decided before writing (fact 8). This would be refuted by evidence that the unit write does not target a specific device offset.
P2: no specific device offset. Justification: Barriers perform only flushes with no bytes written (fact 2). This would be refuted by evidence that a barrier writes bytes to a specific device offset.
P3: specific device offset. Justification: Journal record is written to a specific offset on device 0 (fact 9). This would be refuted by evidence that journal record writes do not target a specific device offset on device 0.
P4: specific device offset. Justification: Journal record is written to a specific offset on device 1 (fact 9). This would be refuted by evidence that journal record writes do not target a specific device offset on device 1.
P5: no specific device offset. Justification: Barriers perform only flushes with no bytes written (fact 2). This would be refuted by evidence that a barrier writes bytes to a specific device offset.
P6: specific device offset. Justification: Root slot is written to a specific offset based on txg modulo 3 (fact 3). This would be refuted by evidence that root slot writes do not target a specific device offset.
P7: specific device offset. Justification: System config slot is written to a specific offset on device 0 (fact 10). This would be refuted by evidence that system config slot writes do not target a specific device offset on device 0.
P8: specific device offset. Justification: System config slot is written to a specific offset on device 1 (fact 10). This would be refuted by evidence that system config slot writes do not target a specific device offset on device 1.

2
P1: undetermined. Justification: Facts do not specify the previous content of unit write locations. This would be refuted by evidence that the previous content of unit write locations is known.
P2: not applicable. Justification: Barriers have no specific device offset, so previous content is irrelevant. This would be refuted by evidence that barriers have a specific device offset.
P3: undetermined. Justification: Facts do not specify the previous content of journal record slots. This would be refuted by evidence that the previous content of journal record slots is known.
P4: undetermined. Justification: Facts do not specify the previous content of journal record slots. This would be refuted by evidence that the previous content of journal record slots is known.
P5: not applicable. Justification: Barriers have no specific device offset, so previous content is irrelevant. This would be refuted by evidence that barriers have a specific device offset.
P6: determined. Justification: Root slot's previous content is the prior root record for that slot (fact 3). This would be refuted by evidence that root slots do not hold prior root records.
P7: determined. Justification: System config slot's previous content is the prior system config for that device (fact 10). This would be refuted by evidence that system config slots do not hold prior system config.
P8: determined. Justification: System config slot's previous content is the prior system config for that device (fact 10). This would be refuted by evidence that system config slots do not hold prior system config.

3
P1: undetermined. Justification: Facts do not specify whether the unit write offset is in use by live data; probe could destroy data or not. This would be refuted by evidence that the offset is not in use or is in use.
P2: safe. Justification: Barriers have no write offset, so no probe write is performed. This would be refuted by evidence that a probe write occurs at a non-existent offset, causing unintended behavior.
P3: undetermined. Justification: Journal slot's previous content is undetermined; probe write could destroy needed data or not. This would be refuted by evidence that the slot's previous content is not needed for replay.
P4: undetermined. Justification: Journal slot's previous content is undetermined; probe write could destroy needed data or not. This would be refuted by evidence that the slot's previous content is not needed for replay.
P5: safe. Justification: Barriers have no write offset, so no probe write is performed. This would be refuted by evidence that a probe write occurs at a non-existent offset, causing unintended behavior.
P6: unsafe. Justification: Root slot contains current valid root record; probe write would corrupt it, leading to recovery failure. This would be refuted by evidence that the root slot's current content is not used for recovery.
P7: unsafe. Justification: System config slot holds current valid config; probe write would overwrite it, causing config corruption. This would be refuted by evidence that the slot being probed does not hold a valid system config.
P8: unsafe. Justification: System config slot holds current valid config; probe write would overwrite it, causing config corruption. This would be refuted by evidence that the slot being probed does not hold a valid system config.
