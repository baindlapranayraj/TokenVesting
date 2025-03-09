import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { BankrunProvider } from "anchor-bankrun";
import { BanksClient, ProgramTestContext } from "solana-bankrun";
import { TokenVesting } from "../target/types/token_vesting";

export interface IContextAccount {
  keypair: anchor.web3.Keypair;
  provider: BankrunProvider;
  program: Program<TokenVesting>;
}

interface Context {
  context: ProgramTestContext;
  client: BanksClient;
  program: Program<TokenVesting>;
  provider: BankrunProvider;
}

interface Keypair {
  employer: IContextAccount;
  employee: IContextAccount;
}

interface Accounts {
  grantPDA: PublicKey;
  grantShecdulePDA: PublicKey;
  mintAccount: PublicKey;
  vaultAccount: PublicKey;
  employerATA: PublicKey;
}

export type TestSetupType = Context & Keypair & Accounts;

export type ClaimTestCaseType = {
  shouldSuccess: boolean;
  des: string;
  slotTime: number;
};
