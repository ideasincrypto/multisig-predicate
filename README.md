## Multisignature Predicate
The code presented in this repository allows native multisignature transactions on Fuel, thanks to the use of predicates. The Fuel modular execution layer is a UTXO-based blockchain, with multiple native asset support. As with Bitcoin, coins can be sent to an address or to the hash of a piece of code, called script in Bitcoin, and predicate within Fuel. To spend a UTXO whose owner is the hash of a predicate, we need to send the predicate code and that predicate has to evaluate to true on the Fuel VM.

### Why build a multisignature predicate?
Multisignature predicates are an important building block for building more complex protocols such as payment channels/networks, as they allow a quick spending path when all the parties involved are cooperating, but also allow native escrow.
As Fuel VM also support contracts, it would be possible to build a contract based multisignature wallet, but predicates are much cheaper as their verification doesn't require gas, thanks to additional constraints on the Fuel VM to avoid node resource exhaustion by attackers.

### Comments on the code
The predicate present in this repository is a 2 out of 3 multisignature predicate: i.e it requires only 2 out of the 3 signatures made by any of the 3 signing addresses embedded in the predicate.
To see the predicate in action, it is possible to run the tests. The tests are checking for the correct behavior in different conditions, including signing in a different order and signing twice with the same key.

### Future Developments
The code in the predicate is a WIP because it is not efficient. It assumes that all witnesses are signatures that are required to verify the predicate, but that doesn't have to be true: a transaction could have a standard pay-to-address coin in addition to a pay-to-hash coin which would require a different strategy. For example, predicates allow predicate data to be passed in the verification and we could pass the witnesses's indexes of right signatures for spending that input as predicate data. In a [separate branch](https://github.com/recizk/multisig-predicate/tree/predicate_data), I have started experimenting with this approach.

Another large source of improvements would require modifications to Fuel itself. Since during predicate verification loops are not allowed, it is impossible to build efficient (in terms of bytecode length) predicates for large multisignature sets. Even the bytecode for the verification of 3 keys is quite long (almost 1000 bytes). Therefore, I believe a specialized opcode, similar to Bitcoin's OP_CHECKMULTISIG, would be helpful to build more efficient multisignature predicates.
