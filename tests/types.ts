import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BankrunProvider } from "anchor-bankrun";
import { TokenVesting } from "../target/types/token_vesting";

export interface IContextAccount {
  keypair: anchor.web3.Keypair;
  provider: BankrunProvider;
  program: Program<TokenVesting>;
}
