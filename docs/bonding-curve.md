# Bonding Curve Explained

This document provides a detailed explanation of the bonding curve used in the bonding contract.

## What is a Bonding Curve?

A bonding curve is a mathematical function that defines the relationship between the price and supply of a token. As the supply of tokens increases, the price of each token changes according to the curve's formula.

## Quadratic Bonding Curve

The bonding contract uses a quadratic bonding curve, where the price is determined by the formula:

```
price = reserve / (supply^2)
```

This creates a smooth curve that becomes more expensive as the supply increases.

## How It Works

### Initial State

The contract is initialized with an initial supply and reserve:

- Initial Supply: 1,000,000 tokens
- Initial Reserve: 1,000,000 diesel

The initial price is calculated as:

```
price = 1,000,000 / (1,000,000^2) = 1 / 1,000,000 = 0.000001 diesel per token
```

### Buying Tokens

When a user buys tokens with diesel, the contract:

1. Calculates the amount of tokens to mint based on the bonding curve
2. Updates the reserve by adding the diesel received
3. Updates the supply by adding the tokens minted
4. Updates the buyer's balance
5. Returns the minted tokens to the buyer

The price for buying tokens is calculated using the integral of the price curve:

```
price = reserve * ((supply + amount)^2 - supply^2) / supply^2
```

This ensures that the price increases as more tokens are minted.

### Selling Tokens

When a user sells tokens, the contract:

1. Calculates the amount of diesel to return based on the bonding curve
2. Updates the reserve by subtracting the diesel returned
3. Updates the supply by subtracting the tokens received
4. Returns the diesel to the seller

The price for selling tokens is calculated using the integral of the price curve:

```
price = reserve * (supply^2 - (supply - amount)^2) / supply^2
```

This ensures that the price decreases as tokens are burned.

## Slippage

Due to the nature of the bonding curve, there is slippage when buying and selling tokens. This means that the price per token when buying is higher than the price per token when selling.

For example:

1. User buys 10,000 tokens with 100 diesel
   - Price per token: 0.01 diesel
2. User immediately sells 10,000 tokens
   - Receives less than 100 diesel due to slippage
   - Price per token: < 0.01 diesel

This slippage is a fundamental property of bonding curves and is not a bug. It ensures that the contract always has enough reserve to buy back tokens.

## Price Impact

The price impact is the change in price caused by a trade. The larger the trade relative to the reserve, the larger the price impact.

For example:

1. Current price: 0.001 diesel per token
2. User buys 100,000 tokens with 1,000 diesel
3. New price: 0.0015 diesel per token
4. Price impact: 50%

To minimize price impact, users can split large trades into smaller ones.

## Conclusion

The quadratic bonding curve provides a smooth price curve that becomes more expensive as the supply increases. This creates a market for the token where the price is determined by supply and demand.

The bonding contract implements this curve in a way that ensures the contract always has enough reserve to buy back tokens, while also providing a mechanism for price discovery.
