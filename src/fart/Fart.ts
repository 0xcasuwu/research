import {
  Address,
  Blockchain,
  BytesWriter,
  Calldata,
  encodeSelector,
  OP_NET,
  Selector,
  StoredU256,
  StoredAddress,
  ADDRESS_BYTE_LENGTH,
  DeployableOP_20,
  OP20InitParameters,
} from '@btc-vision/btc-runtime/runtime';
import { u256 } from '@btc-vision/as-bignum/assembly';
import { SafeMath } from '@btc-vision/btc-runtime/runtime/types/SafeMath';
import { Revert } from '@btc-vision/btc-runtime/runtime/types/Revert';
import { sha256 } from '@btc-vision/btc-runtime/runtime/env/global';

// Define storage pointers at the top level
const remainingSupplyPointer: u16 = Blockchain.nextPointer;
const totalClaimersPointer: u16 = Blockchain.nextPointer;
const firstClaimerPointer: u16 = Blockchain.nextPointer;
const lastClaimerPointer: u16 = Blockchain.nextPointer;
const balancesPointer: u16 = Blockchain.nextPointer;
const claimedPointer: u16 = Blockchain.nextPointer;

@final
export class Fart extends DeployableOP_20 {
  public readonly NAME: string = 'Fart';

  // State variables initialized in constructor
  protected _remainingSupply: StoredU256;
  protected _totalClaimers: StoredU256;
  protected _firstClaimer: StoredAddress;
  protected _lastClaimer: StoredAddress;

  // Getters for state variables
  public get remainingSupply(): u256 {
    return this._remainingSupply.value;
  }

  public get totalClaimers(): u256 {
    return this._totalClaimers.value;
  }

  constructor() {
    super(null);
    this._remainingSupply = new StoredU256(remainingSupplyPointer, u256.Zero, u256.Zero);
    this._totalClaimers = new StoredU256(totalClaimersPointer, u256.Zero, u256.Zero);
    this._firstClaimer = new StoredAddress(firstClaimerPointer, Address.dead());
    this._lastClaimer = new StoredAddress(lastClaimerPointer, Address.dead());
  }

  public override onDeployment(_calldata: Calldata): void {
    const maxSupply: u256 = SafeMath.mul(u256.from(100000), u256.from(1000000000000000000)); // 100,000 FART
    const decimals: u8 = 18;
    const name: string = 'Fart Token';
    const symbol: string = 'FART';

    this.instantiate(new OP20InitParameters(maxSupply, decimals, name, symbol), true);
    this._remainingSupply.value = maxSupply;
  }

  public override execute(method: Selector, calldata: Calldata): BytesWriter {
    switch (method) {
      case encodeSelector('claim'):
        return this.claim();
      case encodeSelector('claimStats'):
        return this.getClaimStats();
      default:
        return super.execute(method, calldata);
    }
  }

  private claim(): BytesWriter {
    const claimer = Blockchain.tx.sender;
    
    if (this._remainingSupply.value == u256.Zero) {
      throw new Revert('No tokens left to claim');
    }

    // Initialize first claimer if not set
    if (this._totalClaimers.value == u256.Zero) {
      this._firstClaimer.value = claimer;
    }

    // Check if already claimed
    const claimed = this.getClaimStatus(claimer);
    if (claimed.value > u256.Zero) {
      throw new Revert('Already claimed');
    }

    // Update claim status
    claimed.value = u256.One;

    // Mint tokens to claimer (using protected _mint with onlyOwner=false)
    const claimAmount = u256.from(1000000000000000000); // 1 FART
    this._mint(claimer, claimAmount, false); // Important: pass false to bypass owner check

    // Update state
    this._remainingSupply.value = SafeMath.sub(this._remainingSupply.value, claimAmount);
    this._totalClaimers.value = SafeMath.add(this._totalClaimers.value, u256.One);
    this._lastClaimer.value = claimer;

    const response = new BytesWriter(1);
    response.writeBoolean(true);
    return response;
  }

  private getClaimStats(): BytesWriter {
    const writer = new BytesWriter(32 + 32 + 32 + 32);
    writer.writeU256(this._totalClaimers.value);
    writer.writeU256(this._remainingSupply.value);
    writer.writeAddress(this._firstClaimer.value);
    writer.writeAddress(this._lastClaimer.value);
    return writer;
  }

  private getBalance(address: Address): StoredU256 {
    const writer = new BytesWriter(32);
    writer.writeAddress(address);
    return new StoredU256(balancesPointer, u256.fromBytes(writer.getBuffer(), true), u256.Zero);
  }

  private getClaimStatus(address: Address): StoredU256 {
    const writer = new BytesWriter(32);
    writer.writeAddress(address);
    return new StoredU256(claimedPointer, u256.fromBytes(writer.getBuffer(), true), u256.Zero);
  }
}