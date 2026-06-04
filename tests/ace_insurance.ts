import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AceInsurance } from "../target/types/ace_insurance";

describe("ace_insurance", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.aceInsurance as Program<AceInsurance>;

  it("loads the program", () => {
    anchor.getProvider();
    if (!program.programId) {
      throw new Error("Program ID missing");
    }
  });
});
