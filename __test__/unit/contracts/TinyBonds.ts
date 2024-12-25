import { Address, BinaryReader, BinaryWriter } from '@btc-vision/transaction';
import { BytecodeManager, ContractDetails, ContractRuntime } from '@btc-vision/unit-test-framework';
import { encodeNumericSelector } from './utils';

export type ClaimStats = {
  totalClaimers: bigint;
  remainingSupply: bigint;
  firstClaimer: Address;
  lastClaimer: Address;
};

export class TinyBonds extends ContractRuntime {
  // Selectors
  protected readonly purchaseBondSelector = encodeNumericSelector('purchaseBond');
  protected readonly redeemBondSelector = encodeNumericSelector('redeemBond');
  protected readonly redeemBondBatchSelector = encodeNumericSelector('redeemBondBatch');
  protected readonly transferBondSelector = encodeNumericSelector('transferBond');
  protected readonly spotPriceSelector = encodeNumericSelector('spotPrice');
  protected readonly getAmountOutSelector = encodeNumericSelector('getAmountOut');
  protected readonly positionCountOfSelector = encodeNumericSelector('positionCountOf');
  protected readonly setVirtualInputReservesSelector = encodeNumericSelector('setVirtualInputReserves');
  protected readonly setVirtualOutputReservesSelector = encodeNumericSelector('setVirtualOutputReserves');
  protected readonly setHalfLifeSelector = encodeNumericSelector('setHalfLife');
  protected readonly setLevelBipsSelector = encodeNumericSelector('setLevelBips');
  protected readonly setLastUpdateSelector = encodeNumericSelector('setLastUpdate');
  protected readonly setPauseSelector = encodeNumericSelector('setPause');
  protected readonly updatePricingSelector = encodeNumericSelector('updatePricing');
  protected readonly halfLifeSelector = encodeNumericSelector('halfLife');
  protected readonly lastUpdateSelector = encodeNumericSelector('lastUpdate');
  protected readonly levelBipsSelector = encodeNumericSelector('levelBips');
  protected readonly virtualOutputReservesSelector = encodeNumericSelector('virtualOutputReserves');
  protected readonly virtualInputReservesSelector = encodeNumericSelector('virtualInputReserves');
  protected readonly inputTokenSelector = encodeNumericSelector('inputToken');
  protected readonly outputTokenSelector = encodeNumericSelector('outputToken');

  constructor(details: ContractDetails) {
    super(details);
    this.preserveState();
  }

  private async getResponse(
    buf: Uint8Array,
    sender?: Address,
    origin?: Address,
  ): Promise<Uint8Array> {
    const result = await this.execute(Buffer.from(buf), sender, origin);

    const response = result.response;
    if (response == null) {
      const errorMessage = result.error ? result.error.message : 'Unknown error occurred';
      throw new Error(errorMessage);
    }

    return response;
  }

  public async purchaseBond(to: Address, amountIn: bigint, minOutput: bigint): Promise<{
    sender: Address;
    amountIn: bigint;
    output: bigint;
  }> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.purchaseBondSelector);
    calldata.writeAddress(to);
    calldata.writeU256(amountIn);
    calldata.writeU256(minOutput);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);

    return {
      sender: reader.readAddress(),
      amountIn: reader.readU256(),
      output: reader.readU256(),
    };
  }

  public async redeemBond(to: Address, bondId: bigint): Promise<{
    sender: Address;
    bondId: bigint;
    output: bigint;
  }> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.redeemBondSelector);
    calldata.writeAddress(to);
    calldata.writeU256(bondId);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);

    return {
      sender: reader.readAddress(),
      bondId: reader.readU256(),
      output: reader.readU256(),
    };
  }

  public async redeemBondBatch(to: Address, bondIds: bigint[]): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.redeemBondBatchSelector);
    calldata.writeAddress(to);
    calldata.writeU256(BigInt(bondIds.length));
    
    for (const bondId of bondIds) {
      calldata.writeU256(bondId);
    }

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async transferBond(to: Address, bondId: bigint): Promise<{
    from: Address;
    to: Address;
    sourceBondId: bigint;
    newBondId: bigint;
  }> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.transferBondSelector);
    calldata.writeAddress(to);
    calldata.writeU256(bondId);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);

    return {
      from: reader.readAddress(),
      to: reader.readAddress(),
      sourceBondId: reader.readU256(),
      newBondId: reader.readU256(),
    };
  }

  public async spotPrice(): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.spotPriceSelector);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async getAmountOut(amountIn: bigint): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.getAmountOutSelector);
    calldata.writeU256(amountIn);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async positionCountOf(owner: Address): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.positionCountOfSelector);
    calldata.writeAddress(owner);

    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async setVirtualInputReserves(newValue: bigint): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.setVirtualInputReservesSelector);
    calldata.writeU256(newValue);
    await this.getResponse(calldata.getBuffer());
  }

  public async setVirtualOutputReserves(newValue: bigint): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.setVirtualOutputReservesSelector);
    calldata.writeU256(newValue);
    await this.getResponse(calldata.getBuffer());
  }

  public async setHalfLife(newValue: bigint): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.setHalfLifeSelector);
    calldata.writeU256(newValue);
    await this.getResponse(calldata.getBuffer());
  }

  public async setLevelBips(newValue: bigint): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.setLevelBipsSelector);
    calldata.writeU256(newValue);
    await this.getResponse(calldata.getBuffer());
  }

  public async setLastUpdate(): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.setLastUpdateSelector);
    await this.getResponse(calldata.getBuffer());
  }

  public async setPause(): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.setPauseSelector);
    await this.getResponse(calldata.getBuffer());
  }

  public async updatePricing(
    newVirtualInput: bigint,
    newVirtualOutput: bigint,
    newHalfLife: bigint,
    newLevelBips: bigint,
    lastUpdateNow: boolean,
    pause: boolean
  ): Promise<void> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.updatePricingSelector);
    calldata.writeU256(newVirtualInput);
    calldata.writeU256(newVirtualOutput);
    calldata.writeU256(newHalfLife);
    calldata.writeU256(newLevelBips);
    calldata.writeBoolean(lastUpdateNow);
    calldata.writeBoolean(pause);
    await this.getResponse(calldata.getBuffer());
  }

  public async halfLife(): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.halfLifeSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async lastUpdate(): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.lastUpdateSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async levelBips(): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.levelBipsSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async virtualOutputReserves(): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.virtualOutputReservesSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async virtualInputReserves(): Promise<bigint> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.virtualInputReservesSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readU256();
  }

  public async inputToken(): Promise<Address> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.inputTokenSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readAddress();
  }

  public async outputToken(): Promise<Address> {
    const calldata = new BinaryWriter();
    calldata.writeSelector(this.outputTokenSelector);
    const response = await this.getResponse(calldata.getBuffer());
    const reader = new BinaryReader(response);
    return reader.readAddress();
  }

  protected defineRequiredBytecodes(): void {
    BytecodeManager.loadBytecode(`./build/TinyBonds.wasm`, this.address);
  }
}