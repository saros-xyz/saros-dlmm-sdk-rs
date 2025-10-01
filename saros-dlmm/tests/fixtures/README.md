# Solana test fixtures

```
solana program dump [program_id] [filename.so]
```

## Simulation test example based on Invariant AMM

#### Make sure you're on the right network, `mainnet-beta`.

```
solana config set --url https://your-own-rpc.com
```

#### Create a snapshot for our `INVARIANT_SAROS_USDC` pool in `/saros-dlmm`

```
cargo run -- --rpc-url <rpc_url> snapshot-amm --amm-id <amm_id>
cargo run -- --rpc-url https://api.devnet.solana.com snapshot-amm --amm-id FvKuEuRyfDZ8catHJznC7heKLkC1uopRaaKMDY1Nym2T
```

You should see a new `ADPKeitAZsAeRJfhG2GoDrZENB3xt9eZmggkj7iAXY78` folder being created in `/tests/fixtures/accounts`.

_\* If you get this error "No in amount for mint", add an entry to `TOKEN_MINT_TO_IN_AMOUNT` in `test_harness.rs`, then run the snapshot again._

#### Dump the program into `/saros-dlmm/tests/fixtures`

```
solana program dump <program_id> <filename>.so
solana program dump 1qbkdrr3z4ryLA7pZykqxvxWPoeifcVKo6ZG9CfkvVE saros_dlmm.so
```
