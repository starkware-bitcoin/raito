# Assumevalid program

Bitcoin Core daemon has an option `-assumevalid` which allows to specify the hash of the block that is considered valid so that it is safe to skip signature checking prior to it.

Default value (May 2025) is the block at height 886157 (almost 3mo old). With Raito we can provide a proof that a more recent block:
1. Traces back to genesis
2. Has a sufficient number of blocks built on top (to ensure finality with large enough probability)

Those two facts together should give a certain confidence that this block is part of the main chain and therefore all the previous ones belong to the main chain and hence are implicitly valid (otherwise miners would not build on top).

## Hot to run the proving process

Use `scripts/data/prove_pow.py`:
```
usage: prove_pow.py [-h] [--start START] [--blocks BLOCKS] [--step STEP] [--verbose]

options:
  -h, --help       show this help message and exit
  --start START    Start block height (if not set, will auto-detect from last proof)
  --blocks BLOCKS  Number of blocks to process
  --step STEP      Step size for block processing
  --verbose        Verbose logging

```

To prove first 5000 block headers in batches of 500 per recursion step:
```
./prove_pow.py --blocks 5000 --step 500 --verbose
``` 