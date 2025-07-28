<div align="center">
    <img src="./docs/img/raito_logo.png" alt="raito-logo" height="260"/>

   ***Bitcoin ZK client written in Cairo.***

<a href="https://github.com/keep-starknet-strange/raito/actions/workflows/build.yml"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/keep-starknet-strange/raito/check.yml?style=for-the-badge" height=30></a>
<a href="https://bitcoin.org/"> <img alt="Bitcoin" src="https://img.shields.io/badge/Bitcoin-000?style=for-the-badge&logo=bitcoin&logoColor=white" height=30></a>
<a href="https://www.cairo-lang.org/"> <img alt="Cairo" src="https://img.shields.io/badge/Cairo-000?style=for-the-badge" height=30></a>
<a href="https://exploration.starkware.co/"><img src="https://img.shields.io/badge/Exploration Team-000.svg?&style=for-the-badge&logo=data:image/svg%2bxml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iVVRGLTgiPz48c3ZnIGlkPSJhIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxODEgMTgxIj48ZGVmcz48c3R5bGU+LmJ7ZmlsbDojZmZmO308L3N0eWxlPjwvZGVmcz48cGF0aCBjbGFzcz0iYiIgZD0iTTE3Ni43Niw4OC4xOGwtMzYtMzcuNDNjLTEuMzMtMS40OC0zLjQxLTIuMDQtNS4zMS0xLjQybC0xMC42MiwyLjk4LTEyLjk1LDMuNjNoLjc4YzUuMTQtNC41Nyw5LjktOS41NSwxNC4yNS0xNC44OSwxLjY4LTEuNjgsMS44MS0yLjcyLDAtNC4yN0w5Mi40NSwuNzZxLTEuOTQtMS4wNC00LjAxLC4xM2MtMTIuMDQsMTIuNDMtMjMuODMsMjQuNzQtMzYsMzcuNjktMS4yLDEuNDUtMS41LDMuNDQtLjc4LDUuMThsNC4yNywxNi41OGMwLDIuNzIsMS40Miw1LjU3LDIuMDcsOC4yOS00LjczLTUuNjEtOS43NC0xMC45Ny0xNS4wMi0xNi4wNi0xLjY4LTEuODEtMi41OS0xLjgxLTQuNCwwTDQuMzksODguMDVjLTEuNjgsMi4zMy0xLjgxLDIuMzMsMCw0LjUzbDM1Ljg3LDM3LjNjMS4zNiwxLjUzLDMuNSwyLjEsNS40NCwxLjQybDExLjQtMy4xMSwxMi45NS0zLjYzdi45MWMtNS4yOSw0LjE3LTEwLjIyLDguNzYtMTQuNzYsMTMuNzNxLTMuNjMsMi45OC0uNzgsNS4zMWwzMy40MSwzNC44NGMyLjIsMi4yLDIuOTgsMi4yLDUuMTgsMGwzNS40OC0zNy4xN2MxLjU5LTEuMzgsMi4xNi0zLjYsMS40Mi01LjU3LTEuNjgtNi4wOS0zLjI0LTEyLjMtNC43OS0xOC4zOS0uNzQtMi4yNy0xLjIyLTQuNjItMS40Mi02Ljk5LDQuMyw1LjkzLDkuMDcsMTEuNTIsMTQuMjUsMTYuNzEsMS42OCwxLjY4LDIuNzIsMS42OCw0LjQsMGwzNC4zMi0zNS43NHExLjU1LTEuODEsMC00LjAxWm0tNzIuMjYsMTUuMTVjLTMuMTEtLjc4LTYuMDktMS41NS05LjE5LTIuNTktMS43OC0uMzQtMy42MSwuMy00Ljc5LDEuNjhsLTEyLjk1LDEzLjg2Yy0uNzYsLjg1LTEuNDUsMS43Ni0yLjA3LDIuNzJoLS42NWMxLjMtNS4zMSwyLjcyLTEwLjYyLDQuMDEtMTUuOGwxLjY4LTYuNzNjLjg0LTIuMTgsLjE1LTQuNjUtMS42OC02LjA5bC0xMi45NS0xNC4xMmMtLjY0LS40NS0xLjE0LTEuMDgtMS40Mi0xLjgxbDE5LjA0LDUuMTgsMi41OSwuNzhjMi4wNCwuNzYsNC4zMywuMTQsNS43LTEuNTVsMTIuOTUtMTQuMzhzLjc4LTEuMDQsMS42OC0xLjE3Yy0xLjgxLDYuNi0yLjk4LDE0LjEyLTUuNDQsMjAuNDYtMS4wOCwyLjk2LS4wOCw2LjI4LDIuNDYsOC4xNiw0LjI3LDQuMTQsOC4yOSw4LjU1LDEyLjk1LDEyLjk1LDAsMCwxLjMsLjkxLDEuNDIsMi4wN2wtMTMuMzQtMy42M1oiLz48L3N2Zz4=" alt="Exploration Team" height="30"></a>
</div>

## Overview

Raito is a Bitcoin consensus client written in Cairo: it implements the same block validation logic as Bitcoin Core but in a provable language. What that means is after you run block validation you can present the result (e.g. chain state) and a succinct proof of execution correctness. The key is that the other party is no longer required to re-run the validation to ensure the chain state is correct, it is enough to just verify the proof.

Raito is heavily inspired by [ZeroSync](https://github.com/ZeroSync/ZeroSync) project.

> **Disclaimer:** This project is in the early stages of development and should not be used in production. It will evolve rapidly, expect breaking changes.

<p align="center" width="100%">
  <img src="./docs/img/client.svg" alt="client"/>
</p>

At its core, consensus client accepts two inputs: a batch of consecutive blocks <i>n</i> to <i>m</i> and a STARK proof of the state of the chain up to block <i>n−1</i>. It ensures that the historical chain state is valid by verifying the STARK proof. Then, it produces a new chain state by applying the new blocks on top of the historical state. As a result, a proof of the new state is generated.

## Applications

### Bootstrapping full nodes

Raito essentially "compresses" the block validation and hence its primary application is enabling quick node synchronization (aka initial block download — IBD). Currently if you are bootstrapping a new full node from scratch there are two options:

- Fetch all the block headers and transaction from P2P and then apply them one by one, computing the state and accumulating the UTXO set (There is an optimization [`assumevalid`](https://bitcoincore.org/en/2017/03/08/release-0.14.0/#assumed-valid-blocks) that helps to speed up the process);
- Download and import a snapshot of the chain state from some trusted source, run the usual sync in the background (see [`assumeutxo`](https://bitcoinops.org/en/topics/assumeutxo/)).

First option takes a lot of time but it is trustless, while the second one is faster (importing snapshot and catching up with the head still takes time) but at the cost of extra trust assumption that assumeutxo hash is checked for correctness during code review (every time it changes!).  

With STARKs we can enjoy both fast and trust-minimized synchronization:

- You still have to trust that Raito (Bitcoin client), Cairo (compiler toolchain), Stwo (prover toolchain), and STARKs are sound and flawless, i.e. that enough qualified specialists verified the code and math (and issued audit reports!);
- However these are "reusable" trust assumptions: you can produce proof for every new Bitcoin block without a need to review anything (unlike assumeutxo approach); this allows to minimize the gap between the head and the snapshot and that way speed up the IBD.

### Witness aggregation

Non-interactive witness aggregation (also known as [transaction compression](https://delvingbitcoin.org/t/post-quantum-signatures-and-scaling-bitcoin-with-starks/1584)) is a method for more efficient block size utilization. It becomes especially important in the context of migrating to post-quantum signing schemes, which typically involve large public keys and signatures. However, even with the current set of cryptographic primitives, witness aggregation can help address the issue of payment transactions competing with complex contracts and meta-protocols.

How it works: Miners execute blocks and produce a proof of correct execution. Once this proof is available, all transaction witnesses (that include signatures, Taproot scripts, etc) can be discarded, freeing up block space for additional transactions. This approach effectively increases the "effective" block size (i.e., the maximum amount of computation) without increasing bandwidth or storage requirements. It also helps mitigate the impact of large, complex transactions (rollup state settlements, refutations, or inscriptions) that might otherwise postpone the ordinary payment transactions.

### Indexer for meta protocols

Raito can be extended with additional validation logic and custom state tree for protocols like [Runes](https://docs.ordinals.com/runes.html) and be used for bootstrapping meta indexers, similarly to how it is used for bootstrapping Bitcoin full nodes.

### Light clients

A trust-minimized bridge design requires "embedding" a Bitcoin client into the target chain for validating the block headers and verifying transaction inclusion proofs. It is typically a "light" version of the client that is efficient onchain but makes more trust assumptions.

Raito provides building blocks for creating light clients with different tradeoffs. However the job of determining the canonical chain remains outside of Raito's scope and should be implemented separately.

See [UTU relay](https://github.com/lfglabs-dev/utu_relay) for an example how Raito primitives can be utilized to build a one-way bridge.

### L2 solutions

Since Raito compresses block validation and execution it can be leveraged to build Bitcoin scaling solutions aka L2s: aggregate multiple transactions, execute, generate a proof, settle on Bitcoin. It is basically a validity rollup or validium if you choose to post data not on Bitcoin but elsewhere. Same would also work for meta protocols effectively turning them from [sovereign](https://celestia.org/glossary/sovereign-rollup/) to validity rollups.

Such design is much more scalable and secure compared to sidechains and free of some of the limitations of Lightning. The settlement part however remains unsolved and we yet to see proof verification happening in Bitcoin mainnet.

Some interesting read on this problem:

- <https://hackmd.io/@polyhedra/bitcoin>
- <https://l2ivresearch.substack.com/p/recent-progress-on-bitcoin-stark>

### Proof of assets

Consider a problem of proving that you can spend a set of outputs `O` (that are not yet spent as of block `B`) with total amount `>X` without revealing which particular UTXOs these are. This requires several primitives:

- Utreexo accumulator: a hash commitment scheme tailored for UTXO set, allows to produce succinct inclusion proofs given the current roots
- Utreexo enabled ZK client: a provable Bitcoin consensus client which also maintains the UTXO set and updates Utreexo accumulator

Given these primitives we can prove the following statements:

- If you start with genesis and sequentially execute `B` blocks you'd end up with a certain state of Utreexo accumulator `R`
- A set of outputs `O` belongs to the UTXO set at that point
- The total amount of outputs `O` is greater than `X`
- You are able to produce a valid witness to spend all the outputs `O`

Aggregating all above would give us a proof of assets, which can be part of a more complex protocol such as proof of reserves (aka proof of solvency).  
Proof of reserves protocol allows to show that the reserves a crypto exchange hold are actually held and that their clients’ funds are “safe“.

See also:

- <https://delvingbitcoin.org/t/proving-utxo-set-inclusion-in-zero-knowledge/1142>

## Roadmap

<p align="center" width="100%">
  <img src="./docs/img/components.svg" alt="components"/>
</p>

### Milestone 1 - Block header validation

Implement a reduced light client that can verify a range of blocks starting at genesis.  
It does not have to validate execution, just check that the block header fields follow the protocol.

Tasks:

- [x] block hash computation
- [x] proof-of-work validation/computation
- [x] block time validation/computation
- [x] block difficulty adjustment
- [x] script for fetching arbitrary block data
- [x] script for preparing program arguments
- [x] script for running the program e2e for multiple blocks

### Milestone 2 - Partial transaction validation

Extend light client with partial transaction validation, but without UTXO checks.

Tasks:

- [x] reassess validation check list (analyze Bitcoin core codebase)
- [x] generate & run integration tests e2e instead of Cairo codegen
- [x] transaction ID calculation
- [x] transaction root computation
- [x] validate transaction fee
- [x] validate coinbase transaction
- [x] validate that transaction can be mined (locktime, sequence, coinbase maturity)
- [x] validate segwit specific data (wtxid commitment)
- [x] validate block weight
- [x] script that fetches blocks extended with references UTXOs
- [x] script that runs the program e2e for a span of blocks

### Milestone 3 - Bitcoin script validation

Try to run script validation with external Cairo crate.

Tasks:

- [x] Integrate Shinigami-script

### Milestone 4 - UTXO set verification

Add inclusion proofs for the UTXOs included in the block.

Tasks:

- [x] isolate unspendable outputs (OP_RETURN, etc)
- [x] implement cache for UTXOs spent in the same block they are created (*)
- [x] implement transaction outpoint hashing
- [x] implement Utreexo accumulator (addition)
- [x] Utreexo backend that maintains utxo set and Utreexo roots
- [x] implement Utreexo single inclusion proof verification
- [x] implement Utreexo single output removal
- [x] implement Utreexo bridge node that generates individual inclusion proofs
- [x] implement script that runs the program e2e for a span of blocks
- [x] implement Utreexo accumulator version compatible with [rustreexo](https://github.com/mit-dci/rustreexo)

### Milestone 5 - Full consensus validation

Validate full block execution over large number of blocks, including the Bitcoin scripts checks and Utreexo proofs.

- [x] consensus logic
- [ ] consensus logic + utreexo proofs  
- [ ] consensus logic + utreexo proofs + scripts

### Milestone 6 - Proving

Recursively verify STARK proofs of chain state updates. Still largely tbd. From initial observations it is clear that a series of optimizations will be necessary.

- [ ] sha256 optimization
- [x] don't use ByteArray when serializing data
- [ ] blocklevel recursion
- [ ] consider using garaga msm to batch signature verifications
- [ ] identify other Cairo code botlenecks

# Contact

- [Telegram](https://t.me/ShinigamiStarknet)
- [OnlyDust](https://app.onlydust.com/p/raito---bitcoin-zk-client)

## Usage

This will compile all the packages:

```bash
scarb build
```

This will run tests for all the packages:

```bash
scarb test
```

## Build dependencies

Install necessary packages required by Python scripts:

```bash
pip install -r scripts/data/requirements.txt
```

## References

- [Data processing notes](./docs/data.md)
- [Utreexo implementation notes](./docs/utreexo.md)
- [Resource usage](./docs/resource_usage_report.csv) (without sha2 steps!)
- [ZeroSync](https://github.com/ZeroSync/ZeroSync)
- [Bitcoin VM in Cairo](https://github.com/keep-starknet-strange/shinigami)
- [STWO](https://github.com/starkware-libs/stwo)
- [Cairo](https://www.cairo-lang.org/)
- [Circle STARK paper](https://eprint.iacr.org/2024/278)

## Name reference

Raito is a reference to Light Yagami (夜神月, Yagami Raito) from the manga/anime Death Note.

- Raito in Japanese means "Light", which in turns can refer to Lightning ⚡ (and hence both a reference to speed of verification of the Bitcoin blockchain using a ZKP and a reference to the Lightning Network)
- Raito can work in tandem with [Shinigami](https://github.com/keep-starknet-strange/shinigami) that enables verification of Bitcoin Script programs. Raito = Consensus and Raito = Execution. Since Shinigami was named after Ryuk (Shinigami in Death Note), Raito was named after Light (Raito in Death Note).
- What Raito writes in the Death Note always happen, so you can see it as a source of truth, similarly to how you use a Zero-Knowledge Proof to verify the integrity of a computation.

## Contributors ✨

Thanks goes to these wonderful people ([emoji key](https://allcontributors.org/docs/en/emoji-key)):

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/AbdelStark"><img src="https://avatars.githubusercontent.com/u/45264458?v=4?s=100" width="100px;" alt="A₿del ∞/21M 🐺 - 🐱"/><br /><sub><b>A₿del ∞/21M 🐺 - 🐱</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=AbdelStark" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/maciejka"><img src="https://avatars.githubusercontent.com/u/190855?v=4?s=100" width="100px;" alt="Maciej Kamiński @ StarkWare"/><br /><sub><b>Maciej Kamiński @ StarkWare</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=maciejka" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/b-j-roberts"><img src="https://avatars.githubusercontent.com/u/54774639?v=4?s=100" width="100px;" alt="Brandon R"/><br /><sub><b>Brandon R</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=b-j-roberts" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://nodeguardians.io/character/tadev-guardian"><img src="https://avatars.githubusercontent.com/u/122918260?v=4?s=100" width="100px;" alt="Tristan"/><br /><sub><b>Tristan</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=TAdev0" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Jeanmichel7"><img src="https://avatars.githubusercontent.com/u/59661788?v=4?s=100" width="100px;" alt="Jean-Michel"/><br /><sub><b>Jean-Michel</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=Jeanmichel7" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/lomasson"><img src="https://avatars.githubusercontent.com/u/97454276?v=4?s=100" width="100px;" alt="lomasson"/><br /><sub><b>lomasson</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=lomasson" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://nodeguardians.io/character/m-kus"><img src="https://avatars.githubusercontent.com/u/44951260?v=4?s=100" width="100px;" alt="Michael Zaikin"/><br /><sub><b>Michael Zaikin</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=m-kus" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/harsh-ps-2003"><img src="https://avatars.githubusercontent.com/u/119954739?v=4?s=100" width="100px;" alt="Harsh Pratap Singh"/><br /><sub><b>Harsh Pratap Singh</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=harsh-ps-2003" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Xavek"><img src="https://avatars.githubusercontent.com/u/61218841?v=4?s=100" width="100px;" alt="Xavek"/><br /><sub><b>Xavek</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=Xavek" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://www.heyfemi.com/home"><img src="https://avatars.githubusercontent.com/u/53274579?v=4?s=100" width="100px;" alt="Olufemi Olumaiyegun"/><br /><sub><b>Olufemi Olumaiyegun</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=olufemi-olumaiyegun" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/MSghais"><img src="https://avatars.githubusercontent.com/u/59928086?v=4?s=100" width="100px;" alt="MSG"/><br /><sub><b>MSG</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=MSghais" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="http://mubarak23.github.io/"><img src="https://avatars.githubusercontent.com/u/7858376?v=4?s=100" width="100px;" alt="Mubarak Muhammad Aminu"/><br /><sub><b>Mubarak Muhammad Aminu</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=mubarak23" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/oluwa-peski"><img src="https://avatars.githubusercontent.com/u/53274579?v=4?s=100" width="100px;" alt="oluwapeski"/><br /><sub><b>oluwapeski</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=oluwa-peski" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/stevencartavia"><img src="https://avatars.githubusercontent.com/u/112043913?v=4?s=100" width="100px;" alt="Steven"/><br /><sub><b>Steven</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=stevencartavia" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/manlikeHB"><img src="https://avatars.githubusercontent.com/u/109147010?v=4?s=100" width="100px;" alt="Yusuf Habib"/><br /><sub><b>Yusuf Habib</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=manlikeHB" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Gerson2102"><img src="https://avatars.githubusercontent.com/u/71728860?v=4?s=100" width="100px;" alt="Gerson"/><br /><sub><b>Gerson</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=Gerson2102" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/PavitraAgarwal21"><img src="https://avatars.githubusercontent.com/u/85789615?v=4?s=100" width="100px;" alt="PavitraAgarwal21"/><br /><sub><b>PavitraAgarwal21</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=PavitraAgarwal21" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/bloomingpeach"><img src="https://avatars.githubusercontent.com/u/177087057?v=4?s=100" width="100px;" alt="Nguyen Dao"/><br /><sub><b>Nguyen Dao</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=bloomingpeach" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/od-hunter"><img src="https://avatars.githubusercontent.com/u/146340502?v=4?s=100" width="100px;" alt="Hunter001"/><br /><sub><b>Hunter001</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=od-hunter" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/feltroidprime"><img src="https://avatars.githubusercontent.com/u/96737978?v=4?s=100" width="100px;" alt="feltroid Prime"/><br /><sub><b>feltroid Prime</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=feltroidprime" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/mexes20"><img src="https://avatars.githubusercontent.com/u/127276944?v=4?s=100" width="100px;" alt="Mexes"/><br /><sub><b>Mexes</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=mexes20" title="Code">💻</a></td>
    </tr>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/JoE11-y"><img src="https://avatars.githubusercontent.com/u/55321462?v=4?s=100" width="100px;" alt="BlockyJ"/><br /><sub><b>BlockyJ</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=JoE11-y" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Groxan"><img src="https://avatars.githubusercontent.com/u/7966898?v=4?s=100" width="100px;" alt="Maksim Strebkov"/><br /><sub><b>Maksim Strebkov</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=Groxan" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/hwildwood"><img src="https://avatars.githubusercontent.com/u/9042313?v=4?s=100" width="100px;" alt="Herbie Wildwood"/><br /><sub><b>Herbie Wildwood</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=hwildwood" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/fishonamos"><img src="https://avatars.githubusercontent.com/u/43862685?v=4?s=100" width="100px;" alt="Fishon Amos"/><br /><sub><b>Fishon Amos</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=fishonamos" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/raizo07"><img src="https://avatars.githubusercontent.com/u/81079370?v=4?s=100" width="100px;" alt="Wolf"/><br /><sub><b>Wolf</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=raizo07" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/baitcode"><img src="https://avatars.githubusercontent.com/u/801611?v=4?s=100" width="100px;" alt="Ilia Batii"/><br /><sub><b>Ilia Batii</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=baitcode" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Th0rgal"><img src="https://avatars.githubusercontent.com/u/41830259?v=4?s=100" width="100px;" alt="Thomas Marchand"/><br /><sub><b>Thomas Marchand</b></sub></a><br /><a href="https://github.com/keep-starknet-strange/raito/commits?author=Th0rgal" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

This project follows the [all-contributors](https://github.com/all-contributors/all-contributors) specification. Contributions of any kind welcome!
