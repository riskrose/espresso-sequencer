// SPDX-License-Identifier: UNLICENSED
// This contract is auto-generated by gen-demo-contract. Direct edits are not recommended.
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import { BN254 } from "bn254/BN254.sol";
import { IPlonkVerifier as V } from "../src/interfaces/IPlonkVerifier.sol";
import { LightClient as LC } from "../src/LightClient.sol";
import { LightClientMock as LCMock } from "../test/mocks/LightClientMock.sol";

contract CallNewFinalizedState is Script {
    LC.LightClientState public genesis;

    function run(uint32 numBlocksPerEpoch, uint32 numInitValidators, address lcContractAddress)
        external
    {
        uint64 numRegistrations = uint64(3);
        uint64 numExits = uint64(3);

        // Generating a few consecutive states and proofs
        string[] memory cmds = new string[](4);
        cmds = new string[](6);
        cmds[0] = "diff-test";
        cmds[1] = "mock-consecutive-finalized-states";
        cmds[2] = vm.toString(numBlocksPerEpoch);
        cmds[3] = vm.toString(numInitValidators);
        cmds[4] = vm.toString(numRegistrations);
        cmds[5] = vm.toString(numExits);

        bytes memory result = vm.ffi(cmds);
        (LC.LightClientState[] memory states, V.PlonkProof[] memory proofs) =
            abi.decode(result, (LC.LightClientState[], V.PlonkProof[]));

        address admin;
        string memory seedPhrase = vm.envString("MNEMONIC");
        (admin,) = deriveRememberKey(seedPhrase, 0);
        vm.startBroadcast(admin);

        LCMock lc = LCMock(lcContractAddress);

        lc.newFinalizedState(states[0], proofs[0]);

        vm.stopBroadcast();
    }
}
