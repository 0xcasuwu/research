import { Assert, Blockchain, OP_20, opnet, OPNetUnit } from '@btc-vision/unit-test-framework';
import { TinyBonds } from '../contracts/TinyBonds';
import { Address, BinaryWriter } from '@btc-vision/transaction';
import { rnd } from '../contracts/configs';

const deployer: Address = rnd();

await opnet('TinyBonds', async (vm: OPNetUnit) => {
  // Set blockchain context before any contract creation
  Blockchain.msgSender = deployer;
  Blockchain.txOrigin = deployer;

  let inputToken: OP_20;
  let outputToken: OP_20;
  let tinyBonds: TinyBonds;
  const inputTokenAddress: Address = rnd();
  const outputTokenAddress: Address = rnd();
  const tinyBondsAddress: Address = rnd();

  vm.beforeEach(async () => {
    console.log('\n=== Test Setup Starting ===');
    
    // Reset blockchain state
    Blockchain.dispose();
    Blockchain.clearContracts();
    await Blockchain.init();

    // Important: Reset msgSender after blockchain init
    Blockchain.msgSender = deployer;
    Blockchain.txOrigin = deployer;

    // Setup input token
    inputToken = new OP_20({
      file: './lib/bytecode/OP20.wasm',
      address: inputTokenAddress,
      decimals: 18,
      deployer,
    });
    Blockchain.register(inputToken);
    await inputToken.init();
    console.log('Input token initialized');

    // Setup output token
    outputToken = new OP_20({
      file: './lib/bytecode/OP20.wasm',
      address: outputTokenAddress,
      decimals: 18,
      deployer,
    });
    Blockchain.register(outputToken);
    await outputToken.init();
    console.log('Output token initialized');

    // Create deployment calldata
    const deployCalldata = new BinaryWriter();
    deployCalldata.writeAddress(inputTokenAddress);
    deployCalldata.writeAddress(outputTokenAddress);
    deployCalldata.writeU256(100n);
    console.log('Deployment calldata created');

    // Setup TinyBonds
    tinyBonds = new TinyBonds({
      address: tinyBondsAddress,
      deployer,
      deploymentCalldata: Buffer.from(deployCalldata.getBuffer()),
    });
    console.log('TinyBonds instance created');

    Blockchain.register(tinyBonds);
    console.log('TinyBonds registered');
    
    await tinyBonds.init();
    console.log('TinyBonds initialized');

    // Mint some tokens to deployer for testing
    await inputToken.mint(deployer, 10000000);
    await outputToken.mint(deployer, 1000000);
    console.log('Tokens minted to deployer');

    console.log('=== Test Setup Complete ===\n');
  });

  vm.afterEach(() => {
    // Only dispose if the contracts exist
    if (tinyBonds) tinyBonds.dispose();
    if (inputToken) inputToken.dispose();
    if (outputToken) outputToken.dispose();
    Blockchain.dispose();
  });

//   await vm.it('Initializes with correct parameters', async () => {
//     // Check token addresses
//     const inputTokenAddr = await tinyBonds.inputToken();
//     const outputTokenAddr = await tinyBonds.outputToken();
//     Assert.expect(inputTokenAddr).toEqualAddress(inputTokenAddress);
//     Assert.expect(outputTokenAddr).toEqualAddress(outputTokenAddress);

//     // Check owner
//     const owner = await tinyBonds.owner();
//     Assert.expect(owner).toEqualAddress(deployer);

//     // Check term blocks
//     const termBlocks = await tinyBonds.termBlocks();
//     Assert.expect(termBlocks).toEqual(100n);

//     // Check initial pricing parameters
//     const halfLife = await tinyBonds.halfLife();
//     const levelBips = await tinyBonds.levelBips();
//     const lastUpdate = await tinyBonds.lastUpdate();
//     const virtualInputReserves = await tinyBonds.virtualInputReserves();
//     const virtualOutputReserves = await tinyBonds.virtualOutputReserves();

//     Assert.expect(halfLife).toEqual(0n);
//     Assert.expect(levelBips).toEqual(0n);
//     Assert.expect(lastUpdate).toEqual(0n);
//     Assert.expect(virtualInputReserves).toEqual(0n);
//     Assert.expect(virtualOutputReserves).toEqual(0n);

//     // Check not paused
//     const paused = await tinyBonds.paused();
//     Assert.expect(paused).toBeFalse();
//   });

  await vm.it('Allows owner to update pricing parameters', async () => {
    // Get and log the owner address
    const newVirtualInput = Blockchain.expandTo18Decimals(1000);
    const newVirtualOutput = Blockchain.expandTo18Decimals(1000);
    const newHalfLife = 100n;
    const newLevelBips = 5000n; // 50%

    await tinyBonds.updatePricing(
      newVirtualInput,
      newVirtualOutput,
      newHalfLife,
      newLevelBips,
      true, // update last update
      false // don't toggle pause
    );

    Assert.expect(await tinyBonds.virtualInputReserves()).toEqual(newVirtualInput);
    Assert.expect(await tinyBonds.virtualOutputReserves()).toEqual(newVirtualOutput);
    Assert.expect(await tinyBonds.halfLife()).toEqual(newHalfLife);
    Assert.expect(await tinyBonds.levelBips()).toEqual(newLevelBips);
  });

  await vm.it('Calculates correct spot price', async () => {
    // Set up initial pricing parameters
    const virtualInput = Blockchain.expandTo18Decimals(1000);
    const virtualOutput = Blockchain.expandTo18Decimals(1000);
    await tinyBonds.updatePricing(
      virtualInput,
      virtualOutput,
      100n, // half life
      5000n, // level bips (50%)
      true,
      false
    );

    const spotPrice = await tinyBonds.spotPrice();
    Assert.expect(spotPrice).toBeGreaterThan(0n);
  });

  await vm.it('Allows purchase of bonds', async () => {
    // Set up pricing parameters
    await tinyBonds.updatePricing(
      Blockchain.expandTo18Decimals(1000),
      Blockchain.expandTo18Decimals(1000),
      100n,
      5000n,
      true,
      false
    );

    // Approve input tokens
    const purchaseAmount = Blockchain.expandTo18Decimals(10);
    await inputToken.approve(deployer, tinyBondsAddress, purchaseAmount);

    // Calculate expected output
    const expectedOutput = await tinyBonds.getAmountOut(purchaseAmount);
    Assert.expect(expectedOutput).toBeGreaterThan(0n);

    // Purchase bond
    const result = await tinyBonds.purchaseBond(deployer, purchaseAmount, 0n);
    Assert.expect(result.amountIn).toEqual(purchaseAmount);
    Assert.expect(result.output).toEqual(expectedOutput);

    // Check position count
    const positions = await tinyBonds.positionCountOf(deployer);
    Assert.expect(positions).toEqual(1n);
  });

  await vm.it('Allows redemption of bonds', async () => {
    // Set up pricing and purchase bond
    await tinyBonds.updatePricing(
      Blockchain.expandTo18Decimals(1000),
      Blockchain.expandTo18Decimals(1000),
      100n,
      5000n,
      true,
      false
    );

    const purchaseAmount = Blockchain.expandTo18Decimals(10);
    await inputToken.approve(deployer, tinyBondsAddress, purchaseAmount);
    await tinyBonds.purchaseBond(deployer, purchaseAmount, 0n);

    // Advance some blocks
    Blockchain.blockNumber += 50n;

    // Redeem bond
    const initialBalance = await outputToken.balanceOf(deployer);
    const result = await tinyBonds.redeemBond(deployer, 0n);
    
    const finalBalance = await outputToken.balanceOf(deployer);
    Assert.expect(finalBalance).toBeGreaterThan(initialBalance);
    Assert.expect(result.output).toEqual(finalBalance - initialBalance);
  });

  await vm.it('Allows transfer of bonds', async () => {
    // Set up pricing and purchase bond
    await tinyBonds.updatePricing(
      Blockchain.expandTo18Decimals(1000),
      Blockchain.expandTo18Decimals(1000),
      100n,
      5000n,
      true,
      false
    );

    const purchaseAmount = Blockchain.expandTo18Decimals(10);
    await inputToken.approve(deployer, tinyBondsAddress, purchaseAmount);
    await tinyBonds.purchaseBond(deployer, purchaseAmount, 0n);

    // Transfer to new address
    const recipient = rnd();
    const result = await tinyBonds.transferBond(recipient, 0n);

    // Verify transfer
    Assert.expect(await tinyBonds.positionCountOf(deployer)).toEqual(0n);
    Assert.expect(await tinyBonds.positionCountOf(recipient)).toEqual(1n);
    Assert.expect(result.from).toEqualAddress(deployer);
    Assert.expect(result.to).toEqualAddress(recipient);
  });

  await vm.it('Handles batch redemptions', async () => {
    // Set up pricing
    await tinyBonds.updatePricing(
      Blockchain.expandTo18Decimals(1000),
      Blockchain.expandTo18Decimals(1000),
      100n,
      5000n,
      true,
      false
    );

    // Purchase multiple bonds
    const purchaseAmount = Blockchain.expandTo18Decimals(10);
    await inputToken.approve(deployer, tinyBondsAddress, purchaseAmount * 3n);
    
    await tinyBonds.purchaseBond(deployer, purchaseAmount, 0n);
    await tinyBonds.purchaseBond(deployer, purchaseAmount, 0n);
    await tinyBonds.purchaseBond(deployer, purchaseAmount, 0n);

    // Advance some blocks
    Blockchain.blockNumber += 50n;

    // Batch redeem
    const initialBalance = await outputToken.balanceOf(deployer);
    const totalOutput = await tinyBonds.redeemBondBatch(deployer, [0n, 1n, 2n]);
    
    const finalBalance = await outputToken.balanceOf(deployer);
    Assert.expect(finalBalance).toBeGreaterThan(initialBalance);
    Assert.expect(totalOutput).toEqual(finalBalance - initialBalance);
  });

  await vm.it('Should properly set input and output tokens during deployment', async () => {
    console.log('\n=== Token Setup Test Starting ===');
    
    const actualInputToken = await tinyBonds.inputToken();
    console.log(`Expected input token: ${inputTokenAddress.toString()}`);
    console.log(`Actual input token: ${actualInputToken.toString()}`);

    const actualOutputToken = await tinyBonds.outputToken();
    console.log(`Expected output token: ${outputTokenAddress.toString()}`);
    console.log(`Actual output token: ${actualOutputToken.toString()}`);

    Assert.expect(actualInputToken).toEqualAddress(inputTokenAddress);
    Assert.expect(actualOutputToken).toEqualAddress(outputTokenAddress);
    
    console.log('=== Token Setup Test Complete ===\n');
  });
});
