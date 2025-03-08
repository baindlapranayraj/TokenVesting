import {
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
  unpackAccount,
} from "@solana/spl-token";
import {
  Signer,
  Transaction,
  Keypair,
  TransactionInstruction,
  SystemProgram,
  PublicKey,
  AccountInfo,
} from "@solana/web3.js";
import { BanksClient, Clock, ProgramTestContext } from "solana-bankrun";
import { employee, employer, mintKP } from "./constant";

import * as anchor from "@coral-xyz/anchor";

export async function getPDAs(programId: PublicKey) {
  try {
    const [grantPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("grant_account"),
        employer.publicKey.toBuffer(),
        employee.publicKey.toBuffer(),
      ],
      programId
    );

    const [grantShecdule] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("grant_schedule_account"),
        employer.publicKey.toBuffer(),
        employee.publicKey.toBuffer(),
      ],
      programId
    );

    const [grantVault] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_account_seed"), grantPDA.toBuffer()],
      programId
    );
    return { grantPDA, grantShecdule, grantVault };
  } catch (error) {
    console.warn(`🥲 You got an error while try to get PDA: ${error}`);
  }
}

export async function createAssociatedTokenAccount(
  mint: PublicKey,
  owner: Keypair,
  client: BanksClient
) {
  try {
    const ata = getAssociatedTokenAddressSync(mint, owner.publicKey, true);
    const ix = createAssociatedTokenAccountInstruction(
      owner.publicKey,
      ata,
      owner.publicKey,
      mint
    );
    await makeTransaction(client, [ix], [owner]);

    return ata;
  } catch (error) {
    console.error(`🥲 You got an error while creating ATA ${error}`);
  }
}

export async function getMintAccount(payer: Keypair, client: BanksClient) {
  try {
    const rent = await client.getRent();

    const ix = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      lamports: Number(rent.minimumBalance(BigInt(MINT_SIZE))),
      newAccountPubkey: mintKP.publicKey,
      space: MINT_SIZE,
      programId: TOKEN_PROGRAM_ID,
    });

    const mintAccIx = createInitializeMint2Instruction(
      mintKP.publicKey,
      9,
      payer.publicKey,
      payer.publicKey
    );

    await makeTransaction(client, [ix, mintAccIx], [payer, mintKP]);

    return mintKP;
  } catch (error) {
    console.warn(`You got error while trying to get Mint account ${error}`);
  }
}

export async function mintToTokens(
  mint: PublicKey,
  destination: PublicKey,
  amount: number,
  authority: Keypair,
  client: BanksClient
) {
  try {
    const ix = createMintToInstruction(
      mint,
      destination,
      authority.publicKey,
      amount
    );

    return await makeTransaction(client, [ix], [authority]);
  } catch (error) {
    throw new Error(`🥲 you got an error while minting tokens ${error}`);
  }
}

export async function makeTransaction(
  client: BanksClient,
  instructions: TransactionInstruction[],
  signers: Signer[]
) {
  try {
    const trx = new Transaction();
    trx.add(...instructions);
    trx.recentBlockhash = (await client.getLatestBlockhash())[0];
    trx.sign(...signers);

    let metaData = await client.processTransaction(trx);
    // let metaTryProcess = await client.tryProcessTransaction(trx);

    return metaData;
  } catch (error) {
    throw new Error(
      `🥲 You got an error while try to make transaction: ${error}`
    );
  }
}

export async function makeTryProcessTransaction(
  client: BanksClient,
  instructions: TransactionInstruction[],
  signers: Signer[]
) {
  try {
    const trx = new Transaction();
    trx.add(...instructions);
    trx.recentBlockhash = (await client.getLatestBlockhash())[0];
    trx.sign(...signers);

    let metaData = await client.tryProcessTransaction(trx);

    return metaData;
  } catch (error) {
    throw new Error(` ${error}`);
  }
}

export function makeTimeTravle(
  context: ProgramTestContext,
  addedUnixTime: bigint,
  currentClock: Clock
) {
  context.setClock(
    new Clock(
      currentClock.slot,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      currentClock.unixTimestamp + addedUnixTime
    )
  );
}
