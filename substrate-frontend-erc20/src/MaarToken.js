import { Abi, ContractPromise } from '@polkadot/api-contract';
import contractMetadata from './MaarMetadata.json';

const abi = new Abi(contractMetadata);
const addr = '5DtL72WFeFz2EMkvgLZdoXBAAPAujBYo4txSozGc4wK5MpSW';

export default function MaarContract (api) {
  return new ContractPromise(api, abi, addr);
}
