You are reviewing a design decision in a from-scratch copy-on-write filesystem, still in format design, no code yet. Do not use any markdown emphasis anywhere in your answer. Plain English prose and plain lists only.

The situation.

Every data unit on disk carries a plaintext self-describing header naming the logical identity of the data it holds: which tree, which object, which generation, and the offset within the object. The purpose is that after the index is lost, a full-disk scan can pick up each unit and rebuild the mapping from logical identity to physical location.

When encryption is on, the project has a hard rule about the associated data fed to the authenticated encryption: the expected value of that associated data must never come from the plaintext side, and must never be read back from the pointer. The reason is that if the expected value came from the same place as the data being checked, an attacker who moves a whole unit somewhere else would go undetected, because the moved unit carries its own matching plaintext header along with it. The project states plainly that if this rule fails, three of its security claims collapse.

The rebuild path breaks this rule by construction. During rebuild there is no index and no parent pointer, so the only available source for the associated data is the plaintext header of the unit itself. Single field tampering is still caught, because the tag would fail. But moving an entire unit, header and ciphertext together, verifies perfectly, and that whole-unit relocation is exactly what the rule exists to catch.

Two candidate resolutions are on the table. The first is to treat the header as a candidate only, then confirm the placement using a checksum held in the parent pointer. The second is to accept that the rebuild path is a degraded path, and state in writing that the mapping it produces is unauthenticated and requires human confirmation.

Your task. You are the adversarial leg. Attack the second candidate, the one that declares the path degraded.

1. Declaring something a degraded path is a disposition, not a mechanism. What has to exist for that declaration to have any force at all? Name the concrete things that must be true so that a person or a program can later tell that a given filesystem went through this path. Be specific about where such a fact would have to live and what would keep it from being erased.

2. There is an obvious failure mode: the degraded state is announced once at rebuild time and then forgotten, so the filesystem looks normal forever afterwards. Give the sequence of events by which that happens even when everyone acts in good faith. Then say what would prevent it.

3. Who is the human that confirms, and what exactly are they being asked to confirm? Is that a question a person can actually answer? If it is not answerable, say what the requirement really amounts to in practice.

4. Compare against the first candidate honestly. The first candidate is said to be circular, because rebuild happens precisely when the index and its pointers are gone. Is it fully circular, or only partly? Describe any situation where a parent pointer would still be available during a rebuild, and say whether that situation is common enough to matter.

5. Propose what you would actually do. If you think both candidates are inadequate, say what a third option would look like and what it costs.

Be concrete and mechanical. Do not restate the situation. If uncertain, say so and say what would settle it.
