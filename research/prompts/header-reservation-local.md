You are reviewing a design question in a from-scratch copy-on-write filesystem, still in the format design phase with no code yet. Do not use any markdown emphasis anywhere in your answer. Write plain English prose and plain lists.

The situation.

Every data unit on disk carries a plaintext self-describing header. The header currently has five fields, chosen so that a full-disk scan can rebuild the index after it is lost: a unit type tag, a tree id, an object id, an object birth generation, and an anchor offset. The project owner has just said the five fields are probably not enough, and that the header should reserve room for other functions, naming garbage collection as an example.

Two facts constrain the answer. First, the header is paid once per unit, forever, on every unit ever written. Second, the project has a written rule saying that when a field has been shown to be useful, it must not be shrunk or dropped to save space, because what is saved is space and what is lost is the ability of a unit to prove itself in isolation. That same rule explicitly says it is not a licence to add fields freely: whether to add a given field still needs its own justification.

Your task. You are the adversarial leg. Attack the notion of reserving space for future functions.

1. Reserving for the future has a failure mode in both directions. Name the ways reserving too little goes wrong, and the ways reserving too much goes wrong. For each, say what the damage actually is, not just that it is bad. Be specific about which one is recoverable and which one is not.

2. There are at least three different mechanisms for leaving room: naming a concrete field now and leaving it unused, reserving a block of unnamed bytes, and spending one bit in a flags word plus a format version number that allows fields to be appended later. Compare them. For each, say what class of future change it actually rescues and what class it does not. Say which one you would use for a function that is already promised but not yet designed, and which for a function nobody has thought of.

3. Propose a decision rule that separates a justified reservation from a speculative one. The rule must be usable by someone looking at a single proposed field with no other context, and it must be possible to say afterwards whether the rule was followed. Test your own rule against garbage collection as the example: does a garbage collector that scans units and decides which ones are still live need a field in this header, and what exactly would that field be?

4. How would anyone ever find out that the reservation was wrong, in either direction, and how long would it take? If the answer is that nobody finds out until much later, say what measurement or check could shorten that.

Be concrete and mechanical. Do not restate the situation. If you are uncertain, say so and say what would settle it.
