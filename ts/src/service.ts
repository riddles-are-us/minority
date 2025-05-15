import { TxWitness, Service, Event, EventModel, TxStateManager } from "zkwasm-ts-server";
import { IndexedObject, docToJSON, StateObjectModel} from "./info.js";
import { Express } from "express";
import { merkleRootToBeHexString } from "zkwasm-ts-server/src/lib.js";
import mongoose from 'mongoose';

const service = new Service(eventCallback, batchedCallback, extra);
await service.initialize();

let txStateManager = new TxStateManager(merkleRootToBeHexString(service.merkleRoot));

function extra (app: Express) {
  app.get('/data/rounds', async(req:any, res) => {
      const doc = await StateObjectModel.find();
      try {
          const jdoc = doc.map((d) => {
              return docToJSON(d);
          });
          console.log(jdoc);
          res.status(201).send({
              success: true,
              data: jdoc,
          });
      } catch (e) {
          console.log(e);
          res.status(500).send()
      }
  });
}


service.serve();

const EVENT_POSITION_UPDATE = 1;
const EVENT_STATE_UPDATE = 2;

async function bootstrap(merkleRoot: string): Promise<TxWitness[]> {
    /*
       const txs = await txStateManager.getTxFromCommit(merkleRoot);
       console.log("tsx in bootstrap:", txs);
       return txs;
       */
    return [];
}

async function batchedCallback(arg: TxWitness[], preMerkle: string, postMerkle: string) {
    await txStateManager.moveToCommit(postMerkle);
}

async function eventCallback(arg: TxWitness, data: BigUint64Array) {
    console.log("event length ...", data.length);
    if(data.length == 0) {
        return;
    }

    //console.log("eventCallback", arg, data);
    if(data[0] != 0n) {
        console.log("non-zero return, tx failed", data[0]);
        return;
    }
    if(data.length <= 2) {
        console.log("no event data");
        return;
    }

    let event = new Event(data[1], data);
    let doc = new EventModel({
        id: event.id.toString(),
        data: Buffer.from(event.data.buffer)
    });

    try {
        let result = await doc.save();
        if (!result) {
            console.log("failed to save event");
            throw new Error("save event to db failed");
        }
    } catch(e) {
        console.log(e);
        console.log("event ignored");
    }
    let i = 2; // start pos
    while(i < data.length) {
        let eventType = Number(data[i]>>32n);
        let eventLength = data[i]&((1n<<32n)-1n);
        let eventData = data.slice(i+1, i+1+Number(eventLength));
        console.log("event", eventType, eventLength, eventData);
        switch(eventType) {
            case EVENT_POSITION_UPDATE:
                {
                console.log("position event");
            }
            break;
            case EVENT_STATE_UPDATE:
                {
                console.log("indexed object event:");
                let obj = IndexedObject.fromEvent(eventData);
                let doc = await obj.storeObject();
                console.log("indexed object", doc);
            }
            break;
            default:
                console.log("unknown event");
            break;
        }
        i += 1 + Number(eventLength);
    }
}



