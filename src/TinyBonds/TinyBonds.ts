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
    Revert,
    SafeMath,
  } from '@btc-vision/btc-runtime/runtime';
  import { u256 } from '@btc-vision/as-bignum/assembly';
  
  // Define storage pointers at the top level
  const pausedPointer: u16 = Blockchain.nextPointer;
  const initializedPointer: u16 = Blockchain.nextPointer;
  const totalDebtPointer: u16 = Blockchain.nextPointer;
  const ownerPointer: u16 = Blockchain.nextPointer;
  
  // Bond struct equivalent
  class Bond {
    owed: StoredU256;
    redeemed: StoredU256;
    creation: StoredU256;
  
    constructor(owedPointer: u16, redeemedPointer: u16, creationPointer: u16) {
      this.owed = new StoredU256(owedPointer, u256.Zero, u256.Zero);
      this.redeemed = new StoredU256(redeemedPointer, u256.Zero, u256.Zero);
      this.creation = new StoredU256(creationPointer, u256.Zero, u256.Zero);
    }
  }
  
  // Pricing struct equivalent
  class Pricing {
    virtualInputReserves: StoredU256;
    virtualOutputReserves: StoredU256;
    lastUpdate: StoredU256;
    halfLife: StoredU256;
    levelBips: StoredU256;
  
    constructor(
      virtualInputPointer: u16,
      virtualOutputPointer: u16,
      lastUpdatePointer: u16,
      halfLifePointer: u16,
      levelBipsPointer: u16
    ) {
      this.virtualInputReserves = new StoredU256(virtualInputPointer, u256.Zero, u256.Zero);
      this.virtualOutputReserves = new StoredU256(virtualOutputPointer, u256.Zero, u256.Zero);
      this.lastUpdate = new StoredU256(lastUpdatePointer, u256.Zero, u256.Zero);
      this.halfLife = new StoredU256(halfLifePointer, u256.Zero, u256.Zero);
      this.levelBips = new StoredU256(levelBipsPointer, u256.Zero, u256.Zero);
    }
  }
  
  @final
  export class TinyBonds extends OP_NET {
    private paused: StoredBool;
    private initialized: StoredBool;
    private totalDebt: StoredU256;
    private owner: StoredAddress;
    private pricing: Pricing;
    private inputToken: StoredAddress;
    private outputToken: StoredAddress;
    private term: StoredU256;
  
    constructor() {
      super();
      this.paused = new StoredBool(pausedPointer);
      this.initialized = new StoredBool(initializedPointer);
      this.totalDebt = new StoredU256(totalDebtPointer, u256.Zero, u256.Zero);
      this.owner = new StoredAddress(ownerPointer, Address.dead());
      
      // Initialize Pricing struct
      this.pricing = new Pricing(
        Blockchain.nextPointer,
        Blockchain.nextPointer,
        Blockchain.nextPointer,
        Blockchain.nextPointer,
        Blockchain.nextPointer
      );
    }
  
    // Modifiers
    private onlyOwner(): void {
      if (Blockchain.tx.sender != this.owner.value) {
        throw new Revert('Only owner can call this method');
      }
    }
  
    private whenNotPaused(): void {
      if (this.paused.value) {
        throw new Revert('Contract is paused');
      }
    }
  
    public override onDeployment(calldata: Calldata): void {
      if (this.initialized.value) {
        throw new Revert('Already initialized');
      }

      // Read deployment parameters
      const inputTokenAddr = calldata.readAddress();
      const outputTokenAddr = calldata.readAddress();
      const termValue = calldata.readU256();

      // Set initial state
      this.inputToken = new StoredAddress(Blockchain.nextPointer, inputTokenAddr);
      this.outputToken = new StoredAddress(Blockchain.nextPointer, outputTokenAddr);
      this.term = new StoredU256(Blockchain.nextPointer, termValue, u256.Zero);
      
      this.owner.value = Blockchain.tx.sender;
      this.initialized.value = true;
      this.paused.value = true;
    }

    public override execute(method: Selector, calldata: Calldata): BytesWriter {
      switch (method) {
        case encodeSelector('purchaseBond'):
          return this.purchaseBond(calldata);
        case encodeSelector('redeemBond'):
          return this.redeemBond(calldata);
        case encodeSelector('redeemBondBatch'):
          return this.redeemBondBatch(calldata);
        case encodeSelector('transferBond'):
          return this.transferBond(calldata);
        case encodeSelector('setVirtualInputReserves'):
          return this.setVirtualInputReserves(calldata);
        case encodeSelector('setVirtualOutputReserves'):
          return this.setVirtualOutputReserves(calldata);
        case encodeSelector('setHalfLife'):
          return this.setHalfLife(calldata);
        case encodeSelector('setLevelBips'):
          return this.setLevelBips(calldata);
        case encodeSelector('setLastUpdate'):
          return this.setLastUpdate();
        case encodeSelector('setPause'):
          return this.setPause();
        case encodeSelector('updatePricing'):
          return this.updatePricing(calldata);
        case encodeSelector('halfLife'):
          return this.halfLife();
        case encodeSelector('lastUpdate'):
          return this.lastUpdate();
        case encodeSelector('levelBips'):
          return this.levelBips();
        case encodeSelector('virtualOutputReserves'):
          return this.virtualOutputReserves();
        case encodeSelector('virtualInputReserves'):
          return this.virtualInputReserves();
        case encodeSelector('positionCountOf'):
          const address = calldata.readAddress();
          return this.positionCountOf(address);
        case encodeSelector('spotPrice'):
          return this.spotPrice();
        case encodeSelector('getAmountOut'):
          const amountIn = calldata.readU256();
          return this.getPublicAmountOut(amountIn);
        default:
          throw new Revert('Unknown method');
      }
    }

    // Bond Purchase Logic
    private purchaseBond(calldata: Calldata): BytesWriter {
      this.whenNotPaused();

      const to = calldata.readAddress();
      const amountIn = calldata.readU256();
      const minOutput = calldata.readU256();

      if (this.pricing.virtualInputReserves.value == u256.Zero) {
        throw new Revert('Bad pricing');
      }

      const availableDebtAmount = this.availableDebt();
      const output = this.getAmountOut(
        amountIn,
        availableDebtAmount,
        this.pricing.virtualOutputReserves.value,
        this.pricing.virtualInputReserves.value,
        SafeMath.sub(Blockchain.timestamp, this.pricing.lastUpdate.value),
        this.pricing.halfLife.value,
        this.pricing.levelBips.value
      );

      if (output < minOutput) {
        throw new Revert('Minimum output not met');
      }

      if (availableDebtAmount < output) {
        throw new Revert('Bad output');
      }

      // Transfer input tokens from sender to owner
      const inputToken = new DeployableOP_20(this.inputToken.value);
      inputToken.transferFrom(Blockchain.tx.sender, this.owner.value, amountIn);

      // Update state
      this.totalDebt.value = SafeMath.add(this.totalDebt.value, output);
      this.pricing.virtualInputReserves.value = SafeMath.add(
        this.pricing.virtualInputReserves.value,
        amountIn
      );

      // Create new bond
      const bondId = this.getBondCount(to);
      const bond = new Bond(
        Blockchain.nextPointer,
        Blockchain.nextPointer,
        Blockchain.nextPointer
      );
      bond.owed.value = output;
      bond.redeemed.value = u256.Zero;
      bond.creation.value = u256.from(Blockchain.timestamp);
      this.setBond(to, bondId, bond);

      // Emit event data
      const response = new BytesWriter(96);
      response.writeAddress(Blockchain.tx.sender);
      response.writeU256(amountIn);
      response.writeU256(output);
      return response;
    }

    // Bond storage helpers
    private getBondKey(owner: Address, id: u256): u256 {
      const writer = new BytesWriter(64);
      writer.writeAddress(owner);
      writer.writeU256(id);
      return u256.fromBytes(writer.getBuffer(), true);
    }

    private getBondCount(owner: Address): u256 {
      const writer = new BytesWriter(32);
      writer.writeAddress(owner);
      return new StoredU256(Blockchain.nextPointer, u256.fromBytes(writer.getBuffer(), true), u256.Zero).value;
    }

    private setBond(owner: Address, id: u256, bond: Bond): void {
      const key = this.getBondKey(owner, id);
      const countKey = new BytesWriter(32);
      countKey.writeAddress(owner);
      
      const bondCount = new StoredU256(Blockchain.nextPointer, u256.fromBytes(countKey.getBuffer(), true), u256.Zero);
      if (id >= bondCount.value) {
        bondCount.value = SafeMath.add(id, u256.One);
      }
    }

    // Bond Redemption Logic
    private redeemBond(calldata: Calldata): BytesWriter {
      this.whenNotPaused();

      const to = calldata.readAddress();
      const bondId = calldata.readU256();
      const bond = this.getBond(Blockchain.tx.sender, bondId);

      const output = this.getRedeemAmountOut(
        bond.owed.value,
        bond.redeemed.value,
        bond.creation.value
      );

      if (output == u256.Zero) {
        throw new Revert('Bad output');
      }

      // Update state
      this.totalDebt.value = SafeMath.sub(this.totalDebt.value, output);
      bond.redeemed.value = SafeMath.add(bond.redeemed.value, output);

      // Transfer tokens
      const outputToken = new DeployableOP_20(this.outputToken.value);
      outputToken.transfer(to, output);

      // Emit event data
      const response = new BytesWriter(96);
      response.writeAddress(Blockchain.tx.sender);
      response.writeU256(bondId);
      response.writeU256(output);
      return response;
    }

    private redeemBondBatch(calldata: Calldata): BytesWriter {
      this.whenNotPaused();

      const to = calldata.readAddress();
      const bondCount = calldata.readU256();
      let totalOutput = u256.Zero;

      // Process each bond
      for (let i = u256.Zero; i < bondCount; i = SafeMath.add(i, u256.One)) {
        const bondId = calldata.readU256();
        const bond = this.getBond(Blockchain.tx.sender, bondId);
        
        const output = this.getRedeemAmountOut(
          bond.owed.value,
          bond.redeemed.value,
          bond.creation.value
        );

        bond.redeemed.value = SafeMath.add(bond.redeemed.value, output);
        totalOutput = SafeMath.add(totalOutput, output);
      }

      // Update total debt and transfer tokens
      this.totalDebt.value = SafeMath.sub(this.totalDebt.value, totalOutput);
      const outputToken = new DeployableOP_20(this.outputToken.value);
      outputToken.transfer(to, totalOutput);

      // Return total output
      const response = new BytesWriter(32);
      response.writeU256(totalOutput);
      return response;
    }

    // Bond Transfer Logic
    private transferBond(calldata: Calldata): BytesWriter {
      this.whenNotPaused();

      const to = calldata.readAddress();
      const bondId = calldata.readU256();
      
      // Get source bond
      const sourceBond = this.getBond(Blockchain.tx.sender, bondId);
      
      // Create new bond for recipient
      const newBondId = this.getBondCount(to);
      const newBond = new Bond(
        Blockchain.nextPointer,
        Blockchain.nextPointer,
        Blockchain.nextPointer
      );
      
      // Copy bond data
      newBond.owed.value = sourceBond.owed.value;
      newBond.redeemed.value = sourceBond.redeemed.value;
      newBond.creation.value = sourceBond.creation.value;
      
      // Delete source bond and set new bond
      this.deleteBond(Blockchain.tx.sender, bondId);
      this.setBond(to, newBondId, newBond);

      // Emit event data
      const response = new BytesWriter(128);
      response.writeAddress(Blockchain.tx.sender);
      response.writeAddress(to);
      response.writeU256(bondId);
      response.writeU256(newBondId);
      return response;
    }

    // Pricing Management Functions
    private setVirtualInputReserves(calldata: Calldata): BytesWriter {
      this.onlyOwner();
      const newValue = calldata.readU256();
      this.pricing.virtualInputReserves.value = newValue;
      return new BytesWriter(0);
    }

    private setVirtualOutputReserves(calldata: Calldata): BytesWriter {
      this.onlyOwner();
      const newValue = calldata.readU256();
      this.pricing.virtualOutputReserves.value = newValue;
      return new BytesWriter(0);
    }

    private setHalfLife(calldata: Calldata): BytesWriter {
      this.onlyOwner();
      const newValue = calldata.readU256();
      this.pricing.halfLife.value = newValue;
      return new BytesWriter(0);
    }

    private setLevelBips(calldata: Calldata): BytesWriter {
      this.onlyOwner();
      const newValue = calldata.readU256();
      this.pricing.levelBips.value = newValue;
      return new BytesWriter(0);
    }

    private setLastUpdate(): BytesWriter {
      this.onlyOwner();
      this.pricing.lastUpdate.value = u256.from(Blockchain.timestamp);
      return new BytesWriter(0);
    }

    private setPause(): BytesWriter {
      this.onlyOwner();
      this.paused.value = !this.paused.value;
      return new BytesWriter(0);
    }

    // Update Pricing Function
    private updatePricing(calldata: Calldata): BytesWriter {
      this.onlyOwner();
      
      const newVirtualInput = calldata.readU256();
      const newVirtualOutput = calldata.readU256();
      const newHalfLife = calldata.readU256();
      const newLevelBips = calldata.readU256();
      const lastUpdateNow = calldata.readBoolean();
      const pause = calldata.readBoolean();

      // Update virtual input reserves if not max value
      if (newVirtualInput != u256.Max) {
        this.pricing.virtualInputReserves.value = newVirtualInput;
      }

      // Update virtual output reserves if not max value
      if (newVirtualOutput != u256.Max) {
        this.pricing.virtualOutputReserves.value = newVirtualOutput;
      }

      // Update half life if not max value
      if (newHalfLife != u256.Max) {
        this.pricing.halfLife.value = newHalfLife;
      }

      // Update level bips if not max value
      if (newLevelBips != u256.Max) {
        this.pricing.levelBips.value = newLevelBips;
      }

      // Update last update timestamp if requested
      if (lastUpdateNow) {
        this.pricing.lastUpdate.value = u256.from(Blockchain.timestamp);
      }

      // Update pause state if requested
      if (pause) {
        this.paused.value = !this.paused.value;
      }

      return new BytesWriter(0);
    }

    // Helper Functions
    private getBond(owner: Address, id: u256): Bond {
      const key = this.getBondKey(owner, id);
      return new Bond(
        Blockchain.nextPointer,
        Blockchain.nextPointer,
        Blockchain.nextPointer
      );
    }

    private deleteBond(owner: Address, id: u256): void {
      const key = this.getBondKey(owner, id);
      // Reset bond storage to zero values
      const bond = this.getBond(owner, id);
      bond.owed.value = u256.Zero;
      bond.redeemed.value = u256.Zero;
      bond.creation.value = u256.Zero;
    }

    // View Functions
    private availableDebt(): u256 {
      const outputToken = new DeployableOP_20(this.outputToken.value);
      return SafeMath.sub(
        outputToken.balanceOf(Blockchain.tx.origin),
        this.totalDebt.value
      );
    }

    // Pricing Calculations
    private getRedeemAmountOut(owed: u256, redeemed: u256, creation: u256): u256 {
      let elapsed = SafeMath.sub(Blockchain.timestamp, creation);
      if (elapsed > this.term.value) {
        elapsed = this.term.value;
      }
      
      const amount = SafeMath.mulDiv(owed, elapsed, this.term.value);
      return SafeMath.sub(amount, redeemed);
    }

    private getAmountOut(
      input: u256,
      outputReserves: u256,
      virtualOutput: u256,
      virtualInput: u256,
      elapsed: u256,
      halfLife: u256,
      levelBips: u256
    ): u256 {
      const adjustedVirtualInput = this.expToLevel(virtualInput, elapsed, halfLife, levelBips);
      const denominator = SafeMath.add(adjustedVirtualInput, input);
      const numerator = SafeMath.mul(input, SafeMath.add(outputReserves, virtualOutput));
      return SafeMath.div(numerator, denominator);
    }

    public halfLife(): BytesWriter {
      const response = new BytesWriter(32);
      response.writeU256(this.pricing.halfLife.value);
      return response;
    }

    public lastUpdate(): BytesWriter {
      const response = new BytesWriter(32);
      response.writeU256(this.pricing.lastUpdate.value);
      return response;
    }

    public levelBips(): BytesWriter {
      const response = new BytesWriter(32);
      response.writeU256(this.pricing.levelBips.value);
      return response;
    }

    public virtualOutputReserves(): BytesWriter {
      const response = new BytesWriter(32);
      response.writeU256(this.pricing.virtualOutputReserves.value);
      return response;
    }

    public virtualInputReserves(): BytesWriter {
      const response = new BytesWriter(32);
      const info = this.pricing;
      const adjustedReserves = this.expToLevel(
        info.virtualInputReserves.value,
        SafeMath.sub(Blockchain.timestamp, info.lastUpdate.value),
        info.halfLife.value,
        info.levelBips.value
      );
      response.writeU256(adjustedReserves);
      return response;
    }

    public positionCountOf(address: Address): BytesWriter {
      const response = new BytesWriter(32);
      response.writeU256(this.getBondCount(address));
      return response;
    }

    public spotPrice(): BytesWriter {
      const info = this.pricing;
      const adjustedVirtualInput = this.expToLevel(
        info.virtualInputReserves.value,
        SafeMath.sub(Blockchain.timestamp, info.lastUpdate.value),
        info.halfLife.value,
        info.levelBips.value
      );
      
      const price = SafeMath.mulDiv(
        u256.from(1e18),
        adjustedVirtualInput,
        SafeMath.add(this.availableDebt(), info.virtualOutputReserves.value)
      );
      
      const response = new BytesWriter(32);
      response.writeU256(price);
      return response;
    }

    // Add public getAmountOut wrapper
    private getPublicAmountOut(amountIn: u256): BytesWriter {
      const info = this.pricing;
      const output = this.getAmountOut(
        amountIn,
        this.availableDebt(),
        info.virtualOutputReserves.value,
        info.virtualInputReserves.value,
        SafeMath.sub(Blockchain.timestamp, info.lastUpdate.value),
        info.halfLife.value,
        info.levelBips.value
      );
      
      const response = new BytesWriter(32);
      response.writeU256(output);
      return response;
    }

    private expToLevel(x: u256, elapsed: u256, halfLife: u256, levelBips: u256): u256 {
      // z = x >> (elapsed / halfLife)
      let z = x.shr(SafeMath.div(elapsed, halfLife).toUInt32());
      
      // z -= z * (elapsed % halfLife) / halfLife / 2
      const elapsedMod = SafeMath.mod(elapsed, halfLife);
      const reduction = SafeMath.div(SafeMath.mul(z, elapsedMod), SafeMath.mul(halfLife, u256.from(2)));
      z = SafeMath.sub(z, reduction);
      
      // z += (x - z) * levelBips / 10000
      const diff = SafeMath.sub(x, z);
      const addition = SafeMath.div(SafeMath.mul(diff, levelBips), u256.from(10000));
      return SafeMath.add(z, addition);
    }
  } // End of TinyBonds class