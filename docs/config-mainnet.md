# Bonding Contract Configuration (mainnet)

## Contract

- Name: Bonding Contract
- Version: 0.1.0
- Description: A bonding contract for the Alkanes metaprotocol

## Deployment

- Name: BOND (0x424f4e44)
- Symbol: BND (0x424e44)
- Initial Supply: 10000000
- Initial Reserve: 10000000

## Bonding Curve

- Type: quadratic
- Formula: price = reserve / (supply^2)

## Opcodes

- initialize: 0
- buy: 1
- sell: 2
- get_current_price: 3
- get_buy_price: 4
- get_sell_price: 5
- get_buy_amount: 6
- get_sell_amount: 7
- get_name: 99
- get_symbol: 100
- get_total_supply: 101
- get_reserve: 102
