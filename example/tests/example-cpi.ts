import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { ExampleCpi } from "../target/types/example_cpi";

describe("example-cpi", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.ExampleCpi as Program<ExampleCpi>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
