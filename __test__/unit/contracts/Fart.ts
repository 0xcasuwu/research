import { Address, BinaryReader, BinaryWriter } from '@btc-vision/transaction';
import { BytecodeManager, ContractDetails, ContractRuntime } from '@btc-vision/unit-test-framework';
import { encodeNumericSelector } from './utils';
import { u256 } from '@btc-vision/as-bignum';

export type ClaimStats = {
  totalClaimers: bigint;
  remainingSupply: bigint;
  firstClaimer: Address;
  lastClaimer: Address;
};

export class Fart extends ContractRuntime {
  protected readonly claimSelector = encodeNumericSelector('claim');
  protected readonly transferSelector = encodeNumericSelector('transfer');
  protected readonly balanceOfSelector = encodeNumericSelector('balanceOf');
  protected readonly claimStatsSelector = encodeNumericSelector('claimStats');
  protected readonly approveSelector = encodeNumericSelector('approve');
  constructor(details: ContractDetails) {
    super(details);
    this.preserveState();
  }

    /**
   * Helper function I highly recommend copying into every contract interface.
   * It takes care of checking the result / error and returns the returned bytes.
   * Wrap the response in a BinaryReader and read whatever data you need.
   */
    private async getResponse(
        buf: Uint8Array,
        sender?: Address,
        origin?: Address,
      ): Promise<Uint8Array> {
        const result = await this.execute(Buffer.from(buf), sender, origin);
    
        const response = result.response;
        if (response == null) {
          const errorMessage = result.error ? result.error.message : 'Unknown error occured';
          throw new Error(errorMessage);
        }
    
        return response;
  }

  public async claim(): Promise<boolean> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.claimSelector);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readBoolean();
  }

  public async balanceOf(owner: Address): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.balanceOfSelector);
    calldata.writeAddress(owner);
    
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async transfer(to: Address, amount: bigint): Promise<boolean> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.transferSelector);
    calldata.writeAddress(to);
    calldata.writeU256(amount);
    
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readBoolean();
  }

  public async claimStats(): Promise<ClaimStats> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.claimStatsSelector);
    
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    
    return {
      totalClaimers: reader.readU256(),
      remainingSupply: reader.readU256(),
      firstClaimer: reader.readAddress(),
      lastClaimer: reader.readAddress()
    };
  }

  public async approve(spender: Address, amount: bigint): Promise<boolean> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.approveSelector);
    calldata.writeAddress(spender);
    calldata.writeU256(amount);
    
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readBoolean();
  }
  
  protected defineRequiredBytecodes(): void {
    BytecodeManager.loadBytecode(`./build/Fart.wasm`, this.address);
  }
}