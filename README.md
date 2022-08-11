# cypher-cpi

This repository is a CPI crate and client all-in-one, the CPI helpers for the [Cypher](https://github.com/chugach-foundation/) program were automatically generated by [anchor-gen](https://github.com/saber-hq/anchor-gen), a crate for generating Anchor CPI helpers from JSON IDLs, while some other code to facilitate usage from client applications was also added on top.

## Usage

In order to use this crate in your application, all you need to do is add the following line to the relevant `Cargo.toml`

```toml
cypher = { git = "https://github.com/chugach-foundation/cypher-cpi.git" }
```

### Clients

If you're building a client application and will be interested in decoding Cypher's account structures after an RPC call (see [rust_mm_client](https://github.com/chugach-foundation/market-making/tree/master/rust_mm_client), [cypher-liquidator](https://github.com/chugach-foundation/cypher-liquidator.git) or [cypher-interactive](https://github.com/murlokito/cypher-interactive.git) for more concrete examples), you might want to do the following.

In your `Cargo.toml`, import `cypher` with the feature `"client"` (WARNING: May break compilation of on-chain programs, you should use `AccountLoader` for those):

```toml
cypher = { git = "https://github.com/chugach-foundation/cypher-cpi.git", features = [ "client" ] }
```

With this feature enabled you can then do the following:

```rust
use {
    CypherUser,
    cypher::utils::get_zero_copy_account
};

/// get the account from the RPC
let account = get_account();

/// account should be of type `solana-sdk::account::Account`
let cypher_user = get_zero_copy_account::<CypherUser>(account);
```


## Example

This repository contains an example on how to do CPI calls to the [Cypher](https://github.com/chugach-foundation/) program, as it is an example for educational purposes only, it merely contains structures for the relevant anchor instructions that would allow you to call Cypher and does not actually attempt to do anything else on top of that.

## License

Apache 2.0