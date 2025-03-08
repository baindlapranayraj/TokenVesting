import { Program } from "@coral-xyz/anchor";
import {
  getAssociatedTokenAddressSync,
  unpackAccount,
} from "@solana/spl-token";
import { AccountInfo, PublicKey } from "@solana/web3.js";
import { BanksClient } from "solana-bankrun";
import { TokenVesting } from "../target/types/token_vesting";

export async function fetchProgramDerivedAccounts(
  program: Program<TokenVesting>,
  grantPdaAddress: PublicKey,
  grantSchedulePdaAddress: PublicKey
) {
  try {
    return {
      grantAccount: await program.account.grant.fetch(grantPdaAddress),
      grantScheduleAccount: await program.account.grantShecdule.fetch(
        grantSchedulePdaAddress
      ),
    };
  } catch (error) {
    console.error(`Error fetching Program Derived Accounts (PDAs): ${error}`);
    throw error;
  }
}

export function employeeATA(mint: PublicKey, owner: PublicKey) {
  try {
    const ata = getAssociatedTokenAddressSync(mint, owner, true);

    return ata;
  } catch (error) {
    console.error(`you got an error while finding the employeeATA:- ${error}`);
  }
}

export async function getClientATA(client: BanksClient, address: PublicKey) {
  try {
    const ataInfo = await client.getAccount(address);
    let acc = unpackAccount(address, ataInfo as AccountInfo<Buffer>);

    return acc;
  } catch (error) {
    console.error(`Got Error while fetching the ATA from the client`);
  }
}
