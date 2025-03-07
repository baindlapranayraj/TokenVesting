// ===== Accounts we needed for invoking the program =======
//
//- employer and employee accounts
//- employer and employee token accounts(ATA)
//- grant mint account
//- vault Token Account
//- grant and grant-schedule PDAs

import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import {
  BanksClient,
  BanksTransactionMeta,
  ProgramTestContext,
} from "solana-bankrun";
import { TokenVesting } from "../../target/types/token_vesting";
import { bankrunSetup } from "../setup";
import { test, beforeAll, describe, expect } from "@jest/globals";
import { BankrunProvider } from "anchor-bankrun";
import {
  createAssociatedTokenAccount,
  getClientATA,
  getMintAccount,
  getPDAs,
  makeTransaction,
  mintToTokens,
} from "../helper";
import {
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { employer } from "../constant";
import { IContextAccount } from "../types";

describe("Testing first Vesting Smart Contract", () => {
  let context: ProgramTestContext;
  let client: BanksClient;
  let provider: BankrunProvider;

  let program: Program<TokenVesting>;

  let vaultAccount: PublicKey;
  let grantPDA: PublicKey;
  let grantSchedule: PublicKey;
  let mint: PublicKey;

  let employer: IContextAccount;
  let employerATA: PublicKey;

  beforeAll(async () => {
    try {
      await bankrunSetup().then((res) => {
        context = res.context;
        client = res.bankrunClient;
        provider = res.provider;
        program = res.program;
        employer = res.employer;
      });

      await getMintAccount(provider.wallet.payer, client).then((res) => {
        mint = res.publicKey;
      });

      await getPDAs(program.programId).then((res) => {
        vaultAccount = res.grantVault;
        grantPDA = res.grantPDA;
        grantSchedule = res.grantShecdule;
      });

      employerATA = await createAssociatedTokenAccount(
        mint,
        employer.keypair,
        client
      );
    } catch (error) {
      console.warn(
        `🥲 You got an error while trying setup the frist instruction: ${error}`
      );
    }
  });

  test("lets mint some tokens to the employer ATA", async () => {
    try {
      let amount = 10_000 * 10 ** 9;
      let metaRes: BanksTransactionMeta = await mintToTokens(
        mint,
        employerATA,
        amount,
        provider.wallet.payer,
        client
      );

      console.log("📝 Transaction Logs:");
      metaRes.logMessages.forEach((logs, index) => {
        console.log(`[index: ${index + 1}] ${logs}`);
      });

      let fetchATA = await getClientATA(client, employerATA);

      expect(Number(fetchATA.amount)).toEqual(amount);
    } catch (error) {
      console.error(
        `🥲 You got an error while minting tokens in your test-case ${error}`
      );
      throw new Error(error);
    }
  });

  test("lets test our initialize TokenVesting instruction", async () => {
    try {
    } catch (error) {
      console.error(
        `You got an error while testing your first instruction test-case ${error}`
      );
    }
  });
});
