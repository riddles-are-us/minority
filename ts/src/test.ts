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

  console.log("Start run CREATE_PLAYER...");
  await player.runCommand(INSTALL_PLAYER, 0n, []);

  let g = await player.getState();
  console.log("state.", g);

  console.log("Start run buy card ...");
  let nonce = await player.getNonce();
  await player.runCommand(BUY_CARD, nonce, [0n, 1n]);

  console.log("Start run buy card ...");
  nonce = await player.getNonce();
  await player.runCommand(BUY_CARD, nonce, [1n, 2n]);

  console.log("Start run buy card ...");
  nonce = await player.getNonce();
  await player.runCommand(BUY_CARD, nonce, [3n, 3n]);


  g = await player.getState();
  console.log("state.", g);

  console.log("Start run query rounds ...");
  try {
    let data:any = await player.rpc.queryData(`rounds`);
    console.log(data);
  } catch(e) {
    console.log(e);
  }
}

main();
