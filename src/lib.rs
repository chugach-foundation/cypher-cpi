use anchor_lang::prelude::*;

anchor_gen::generate_cpi_interface!(
    idl_path = "idl.json",
    zero_copy(PriceHistory)
);

#[cfg(feature = "mainnet-beta")]
declare_id!("CYPHER79cJLzQ8iyyr6oeizfGgR9YU9NM9oTMPWak5oQ");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("8Z8nDAa98hgdYCS9SyAyAesxE3ZhAq8Qo1E8v2V8VU56");
