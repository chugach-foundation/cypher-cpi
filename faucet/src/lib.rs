use anchor_lang::prelude::*;

anchor_gen::generate_cpi_interface!(idl_path = "idl.json");

#[cfg(feature = "devnet")]
declare_id!("7njrvFJx4NJQvzywv1LdnPwzYYTSh1wWgGL5vkwTUuSS");
#[cfg(not(feature = "devnet"))]
declare_id!("7njrvFJx4NJQvzywv1LdnPwzYYTSh1wWgGL5vkwTUuSS");
