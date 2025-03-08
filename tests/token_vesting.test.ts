import { describe, test, beforeAll, expect } from "@jest/globals";
import { TestSetupType } from "./types";
import { bankrunSetup } from "./setup";
import {
  createAssociatedTokenAccount,
  getMintAccount,
  getPDAs,
  mintToTokens,
  makeTransaction,
  makeTryProcessTransaction,
} from "./helper";
import {
  getClientATA,
  employeeATA,
  fetchProgramDerivedAccounts,
} from "./accounts";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import BN from "bn.js";

// Define a promise and its resolver to track when setup is complete
let setupPromise: Promise<TestSetupType>;
let resolveSetup: (value: TestSetupType) => void;
let setupError: Error | null = null;

// Initialize the promise
setupPromise = new Promise((resolve) => {
  resolveSetup = resolve;
});

describe("token_vesting testings", () => {
  let testSetup: TestSetupType = {} as TestSetupType;

  // Setup runs before all tests but doesn't define the tests
  beforeAll(async () => {
    try {
      // Wait for bankrunSetup to fully resolve
      const setupData = await bankrunSetup();
      testSetup.context = setupData.context;
      testSetup.client = setupData.bankrunClient;
      testSetup.employee = setupData.employee;
      testSetup.employer = setupData.employer;
      testSetup.program = setupData.program;
      testSetup.provider = setupData.provider;

      const pdaData = await getPDAs(testSetup.program.programId);
      testSetup.grantPDA = pdaData.grantPDA;
      testSetup.grantShecdulePDA = pdaData.grantShecdule;
      testSetup.vaultAccount = pdaData.grantVault;

      testSetup.mintAccount = (
        await getMintAccount(testSetup.provider.wallet.payer, testSetup.client)
      ).publicKey;

      testSetup.employerATA = await createAssociatedTokenAccount(
        testSetup.mintAccount,
        testSetup.employer.keypair,
        testSetup.client
      );

      console.log(`Setting up TestSetup is Done ✅`);

      // Resolve the setupPromise with the completed testSetup
      resolveSetup(testSetup);
    } catch (error) {
      console.log(`Error while setting up the test setup: ${error}`);
      setupError = error instanceof Error ? error : new Error(String(error));
      throw error; // Rethrow to make the test fail properly
    }
  });

  // Define the actual test for minting tokens - waits for setup to complete
  describe("Testing first Vesting Smart Contract", () => {
    test("lets mint some tokens to the employer ATA", async () => {
      // Wait for setup to complete before proceeding
      const setup = await setupPromise;
      if (setupError) throw setupError;

      try {
        // Check if provider and wallet are defined before accessing nested properties
        if (!setup.provider || !setup.provider.wallet) {
          throw new Error("Provider or provider.wallet is undefined");
        }

        let amount = 1_000_000 * 10 ** 9;
        let metaRes = await mintToTokens(
          setup.mintAccount,
          setup.employerATA,
          amount,
          setup.provider.wallet.payer,
          setup.client
        );
        //
        // console.log("📝 Transaction Logs:");
        // metaRes.logMessages.forEach((logs, index) => {
        //   console.log(`[index: ${index + 1}] ${logs}`);
        // });
        //
        let fetchATA = await getClientATA(setup.client, setup.employerATA);

        expect(Number(fetchATA.amount)).toEqual(amount);
      } catch (error) {
        console.error(
          `😲 You got an error while minting tokens in your test-case ${error}`
        );
        throw new Error(String(error));
      }
    });

    test("lets test our initialize TokenVesting instruction", async () => {
      // Wait for setup to complete before proceeding
      const setup = await setupPromise;
      if (setupError) throw setupError;

      try {
        // Check if client exists before accessing methods
        if (!setup.client) {
          throw new Error("Client is undefined");
        }

        // 1 Month in Unix Time ≈ 30 days × 24 hours × 60 minutes × 60 seconds = 2,592,000 seconds
        let one_month_unix = 2_592_000;
        let clock = await setup.client.getClock();

        let start_unix = clock.unixTimestamp;
        let cliff_unix = new BN(Number(start_unix) + 3 * one_month_unix);
        let end_unix = new BN(Number(start_unix) + 12 * 2 * one_month_unix);

        const amountDeposite = new BN(1_000_00);

        // first instruction arguments
        let initGrantArg = {
          cliffDate: cliff_unix,
          startDate: new BN(Number(start_unix)),
          endDate: end_unix,
          grantDeposited: amountDeposite,
        };

        // Check if employer and program exist
        if (
          !setup.employer ||
          !setup.employer.program ||
          !setup.employer.keypair
        ) {
          throw new Error(
            "Employer, employer.program, or employer.keypair is undefined"
          );
        }

        let ix = await setup.employer.program.methods
          .initialize(initGrantArg)
          .accountsStrict({
            employer: setup.employer.keypair.publicKey,
            employee: setup.employee.keypair.publicKey,

            employerToken: setup.employerATA,
            grantMint: setup.mintAccount,
            grant: setup.grantPDA,
            grantVault: setup.vaultAccount,
            grantShecdule: setup.grantShecdulePDA,

            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SYSTEM_PROGRAM_ID,
          })
          .signers([setup.employer.keypair])
          .instruction();

        console.log("📝 Transaction Logs for first instruction test-case:");
        let metaRes = await makeTransaction(
          setup.client,
          [ix],
          [setup.employer.keypair]
        );

        // metaRes.logMessages.forEach((logs, index) => {
        //   console.log(`[index: ${index + 1}] ${logs}`);
        // });

        // Check if program is defined before calling fetchProgramDerivedAccounts
        if (!setup.program) {
          throw new Error("Program is undefined");
        }

        const { grantAccount, grantScheduleAccount } =
          await fetchProgramDerivedAccounts(
            setup.program,
            setup.grantPDA,
            setup.grantShecdulePDA
          );

        expect(Number(grantAccount.totalAmountLocked)).toEqual(
          Number(amountDeposite)
        );
        expect(Number(Number(grantScheduleAccount.cliffDate))).toEqual(
          Number(cliff_unix)
        );
        expect(Number(Number(grantScheduleAccount.endDate))).toEqual(
          Number(end_unix)
        );
      } catch (error) {
        throw new Error(
          `You got an error while testing your first instruction test-case ${error}`
        );
      }
    });
  });

  describe("testing claim tokens", () => {
    test("testing the second-instrcution", async () => {
      // Wait for setup to complete before proceeding
      const setup = await setupPromise;
      if (setupError) throw setupError;

      try {
        // Check if employee and program exist before accessing methods
        if (
          !setup.employee ||
          !setup.employee.program ||
          !setup.employee.keypair
        ) {
          throw new Error(
            "Employee, employee.program, or employee.keypair is undefined"
          );
        }

        if (!setup.employer || !setup.employer.keypair) {
          throw new Error("Employer or employer.keypair is undefined");
        }

        let ix = await setup.employee.program.methods
          .claimGrant()
          .accountsStrict({
            employee: setup.employee.keypair.publicKey,
            employer: setup.employer.keypair.publicKey,

            employeeTokenAccount: employeeATA(
              setup.mintAccount,
              setup.employee.keypair.publicKey
            ),
            grantAccount: setup.grantPDA,
            grantScheduleAccount: setup.grantShecdulePDA,
            grantVaultAccount: setup.vaultAccount,
            grantMint: setup.mintAccount,

            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SYSTEM_PROGRAM_ID,
          })
          .signers([setup.employee.keypair])
          .instruction();

        // Verify client exists before calling makeTryProcessTransaction
        if (!setup.client) {
          throw new Error("Client is undefined");
        }

        const metaData = await makeTransaction(
          setup.client,
          [ix],
          [setup.employee.keypair]
        );

        metaData.logMessages.forEach((log, index) => {
          console.log(`Error Logs:- ${log} index: ${index + 1}`);
        });

        console.log("📝 Transaction Logs for second-instrcution test-case:");
      } catch (error) {
        console.error(
          `Got an error while testing the claim instruction:- ${error}`
        );
        throw new Error(error);
      }
    });
  });
});
