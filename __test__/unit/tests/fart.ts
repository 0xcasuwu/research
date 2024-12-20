import { Assert, Blockchain, opnet, OPNetUnit } from '@btc-vision/unit-test-framework';
import { Fart } from '../contracts/Fart';
import { Address } from '@btc-vision/transaction';
import { rnd } from '../contracts/configs';

await opnet('FART Token Fuzz Tests', async (vm: OPNetUnit) => {
  let fart: Fart;

  vm.beforeEach(async () => {
    Blockchain.dispose();
    Blockchain.clearContracts();
    await Blockchain.init();

    fart = new Fart({
      address: rnd(),
      deployer: rnd(),
    });

    Blockchain.register(fart);
    await fart.init();
  });

  await vm.it('Should handle multiple claims in random order', async () => {
    const numUsers = 10;
    const users: Address[] = [];
    
    // Generate random users
    for (let i = 0; i < numUsers; i++) {
      users.push(rnd());
    }

    // Shuffle array for random claim order
    for (let i = users.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [users[i], users[j]] = [users[j], users[i]];
    }

    // Process claims
    for (const user of users) {
      Blockchain.msgSender = user;
      await fart.claim();
      
      const balance = await fart.balanceOf(user);
      Assert.equal(balance, 1000000000000000000n);
    }

    // Verify final state
    const stats = await fart.claimStats();
    Assert.equal(stats.totalClaimers, BigInt(numUsers));
    Assert.equal(stats.remainingSupply, 100000000000000000000000n - BigInt(numUsers) * 1000000000000000000n);
  });

  await vm.it('Should handle rapid consecutive claims from different addresses', async () => {
    const claimBatch = async (start: number, count: number) => {
      for (let i = 0; i < count; i++) {
        const user = rnd();
        Blockchain.msgSender = user;
        await fart.claim();
      }
    };

    // Simulate multiple batches of claims
    await claimBatch(0, 5);
    await claimBatch(5, 3);
    await claimBatch(8, 4);

    const stats = await fart.claimStats();
    Assert.equal(stats.totalClaimers, 12n);
    Assert.equal(stats.remainingSupply, 100000000000000000000000n - 12n * 1000000000000000000n);
  });

  await vm.it('Should maintain consistent state under stress', async () => {
    const users: Address[] = [];
    const claimed = new Set<string>();
    
    // Generate test scenario
    for (let i = 0; i < 20; i++) {
      const user = rnd();
      users.push(user);
      
      // Randomly try double claims
      if (Math.random() > 0.5) {
        users.push(user);
      }
    }

    // Process all claims
    for (const user of users) {
      Blockchain.msgSender = user;
      
      if (claimed.has(user.toString())) {
        // Should reject double claims
        await Assert.throwsAsync(async () => {
          await fart.claim();
        });
        continue;
      }

      await fart.claim();
      claimed.add(user.toString());
      
      const balance = await fart.balanceOf(user);
      Assert.equal(balance, 1000000000000000000n);
    }

    // Verify final state
    const stats = await fart.claimStats();
    Assert.equal(stats.totalClaimers, BigInt(claimed.size));
    Assert.equal(
      stats.remainingSupply, 
      100000000000000000000000n - BigInt(claimed.size) * 1000000000000000000n
    );
  });
});