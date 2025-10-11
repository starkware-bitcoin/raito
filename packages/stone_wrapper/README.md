# Stone wrapper

An adapter program that makes it possible to verify Raito proof with [Stone](https://github.com/starkware-libs/stone-prover) prover.

## Install

To install Stone prover follow the instructions at https://stone-packaging.pages.dev/

## Proving flow

To produce a Stone compatible proof we need to perform two steps:
1. Generate a Stwo proof that can be verified by a Cairo program, provable with Stone; this mostly means using Poseidon instead of Blake2s as commitments/channel has function.
2. Generate a Stone proof of a program that verifies Stwo proof and forwards the verification output; that's what the current package does.
