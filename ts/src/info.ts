import mongoose from 'mongoose';
import { Market } from 'zkwasm-ts-server';

(BigInt.prototype as any).toJSON = function () {
          return this.toString();
};

interface RoundResult {
  winner: bigint;
  pool: bigint;
  total: bigint;
}

function fromData(u64data: bigint[]): RoundResult {
    const winner: bigint = u64data.shift()!;
    const pool: bigint = u64data.shift()!;
    const total: bigint = u64data.shift()!;
    return {
        winner,
        pool,
        total,
    }
}

export function docToJSON(doc: mongoose.Document) {
    console.log("doc...", doc);
    const obj = doc.toObject({
        transform: (_, ret:any) => {
            delete ret._id;
            return ret;
        }
    });
    return obj;
}

export class IndexedObject {
    // token idx
    index: number;
    // 40-character hexadecimal Ethereum address
    data: bigint[];

    constructor(index: number, data: bigint[]) {
        this.index = index;
        this.data = data;
    }

    toObject() {
        return fromData(this.data)
    }

    toJSON() {
      return JSON.stringify(this.toObject());
    }

    static fromEvent(data: BigUint64Array): IndexedObject {
        return new IndexedObject(Number(data[0]),  Array.from(data.slice(1)))
    }

    async storeObject() {
        let obj = this.toObject() as any;
        let doc = await StateObjectModel.findOneAndUpdate({id: this.index}, obj, {upsert: true});
        return doc;
    }
}

// Define the schema for the Token model
const StateObjectSchema = new mongoose.Schema({
    id: { type: Number, required: true, unique: true},
    winner: { type: BigInt, required: true},
    pool: {type: BigInt, required: true},
    total: {type: BigInt, required: true},
});

StateObjectSchema.pre('init', Market.uint64FetchPlugin);

// Create the Token model
export const StateObjectModel = mongoose.model('MarketObject', StateObjectSchema);
