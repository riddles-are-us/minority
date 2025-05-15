import { Player } from "./api.js";
//import { LeHexBN, ZKWasmAppRpc} from "zkwasm-minirollup-rpc";
import { LeHexBN, query, ZKWasmAppRpc} from "zkwasm-ts-server";
import { createAsyncThunk } from '@reduxjs/toolkit';

const INSTALL_PLAYER = 1n;
const WITHDRAW = 2n;
const DEPOSIT = 3n;
const BUY_CARD = 4n;
const CLAIM_REWARD = 5n;

let account = "1234";

const rpc:any = new ZKWasmAppRpc("http://127.0.0.1:3000");
let player = new Player(account, rpc, DEPOSIT, WITHDRAW);

// Function to pause execution for a given duration
function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  const pubkey = new LeHexBN(query(account).pkx).toU64Array();
  console.log(pubkey);

  let r = await player.rpc.queryConfig();
  console.log("config:", r);


  let g = await player.getState();
  console.log("state:", g);

  let rounds = g.player.data.rounds;
  for (const r of rounds) {
      console.log(r);
  }

  let nonce = await player.getNonce();

  console.log("Start run query rounds ...");
  try {
    let data:any = await player.rpc.queryData(`rounds`);
    console.log(data);
  } catch(e) {
    console.log(e);
  }



  await player.runCommand(CLAIM_REWARD, nonce, [BigInt(rounds[0].round)]);


}

main();
