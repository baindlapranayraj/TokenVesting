// Hear we will setup the bankrunClient context and all keypair accounts

import { Program } from "@coral-xyz/anchor";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { BankrunProvider } from "anchor-bankrun";
import { ProgramTestContext, startAnchor } from "solana-bankrun";
import { TokenVesting } from "../target/types/token_vesting";
import IDL from "../target/idl/token_vesting.json";
import { employer, employee } from "./constant";
import { IContextAccount } from "./types";

const PROGRAM_ID = new PublicKey(IDL.address);

export async function bankrunSetup() {
  try {
    // setting up BankContext
    const context = await startAnchor(
      "",
      [
        {
          name: "token_vesting",
          programId: PROGRAM_ID,
        },
      ],
      [
        {
          address: employer.publicKey,
          info: {
            data: Buffer.alloc(0),
            executable: false,
            lamports: LAMPORTS_PER_SOL * 10,
            owner: SYSTEM_PROGRAM_ID,
          },
        },
        {
          address: employee.publicKey,
          info: {
            data: Buffer.alloc(0),
            owner: SYSTEM_PROGRAM_ID,
            lamports: LAMPORTS_PER_SOL * 10,
            executable: false,
          },
        },
      ]
    );

    const provider = new BankrunProvider(context);
    const bankrunClient = context.banksClient;
    const program = new Program<TokenVesting>(IDL as TokenVesting, provider);

    const employeeAccount = getAccount(employee, context);
    const employerAccount = getAccount(employer, context);

    // Verify all critical components exist before returning
    if (
      !context ||
      !bankrunClient ||
      !program ||
      !employeeAccount ||
      !employerAccount
    ) {
      throw new Error("Failed to initialize one or more required components");
    }

    return {
      context,
      bankrunClient,
      provider,
      program,
      employee: employeeAccount,
      employer: employerAccount,
      wallet: provider.wallet,
    };
  } catch (error) {
    console.warn(
      `🥲You got an error while setting up the bankrunSetup:- file-name is setup.ts and error is ${error} 🥲`
    );

    // Return a default object with empty/mock values to prevent undefined errors
    return {
      context: null,
      bankrunClient: null,
      provider: null,
      program: null,
      employee: {
        provider: null,
        program: null,
        keypair: employee,
      },
      employer: {
        provider: null,
        program: null,
        keypair: employer,
      },
      wallet: null,
    };
  }
}

function getAccount(
  keypair: Keypair,
  context: ProgramTestContext
): IContextAccount {
  const provider = new BankrunProvider(context);
  provider.wallet = new NodeWallet(keypair);

  const program = new Program<TokenVesting>(IDL as TokenVesting, provider);

  const account: IContextAccount = {
    provider,
    program,
    keypair,
  };
  return account;
}
// ++++++++++++++++++++Learnings ++++++++++++++++++++
//
// 1) What is startAnchor?
//
// startAnchor is a function (likely from a testing framework) that initializes a mock Solana environment for testing.
// It simulates a Solana blockchain, allowing you to test your programs without needing a real blockchain connection.
