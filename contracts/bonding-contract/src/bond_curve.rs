//! Bond-based liquidity implementation using time-decay pricing
//! 
//! This module implements a bond-based liquidity system inspired by Tiny-Bonds.
//! Users provide diesel and receive bonds that mature over time, allowing them to claim alkane.
//! The price of bonds decreases over time using an exponential decay mechanism.
//! 
//! The pricing mechanism is defined as:
//! - Price = virtual_input / (available_debt + virtual_output)
//! - Virtual input decays exponentially over time based on half-life

use alkanes_support::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Scaling factor used to maintain precision in integer calculations
const SCALING_FACTOR: u128 = 1_000_000;

/// Bond structure representing a user's position
#[derive(Clone, Debug)]
pub struct Bond {
    /// Total amount owed to the bonder
    pub owed: u128,
    /// Amount already redeemed
    pub redeemed: u128,
    /// Timestamp of bond creation
    pub creation: u64,
}

/// Pricing structure for the bond curve
#[derive(Clone, Debug)]
pub struct Pricing {
    /// Virtual input reserves (decays over time)
    pub virtual_input_reserves: u128,
    /// Virtual output reserves (constant)
    pub virtual_output_reserves: u128,
    /// Last update timestamp
    pub last_update: u64,
    /// Half-life in seconds
    pub half_life: u64,
    /// Level in basis points (0-10000)
    pub level_bips: u64,
}

/// Bond curve implementation for time-decay pricing
#[derive(Clone)]
pub struct BondCurve {
    /// Pricing parameters
    pub pricing: Pricing,
    /// Total debt (sum of all unredeemed bonds)
    pub total_debt: u128,
    /// Term for bond maturity in seconds
    pub term: u64,
}

impl BondCurve {
    /// Create a new bond curve with the given parameters
    pub fn new(
        virtual_input_reserves: u128,
        virtual_output_reserves: u128,
        half_life: u64,
        level_bips: u64,
        term: u64,
    ) -> Self {
        let current_time = Self::current_timestamp();
        
        Self {
            pricing: Pricing {
                virtual_input_reserves,
                virtual_output_reserves,
                last_update: current_time,
                half_life,
                level_bips,
            },
            total_debt: 0,
            term,
        }
    }
    
    /// Get the current timestamp in seconds
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Calculate the exponential decay to level
    /// 
    /// # Formula
    /// 
    /// z = x >> (elapsed / half_life)
    /// z -= z * (elapsed % half_life) / half_life >> 1
    /// z += (x - z) * level_bips / 10000
    pub fn exp_to_level(x: u128, elapsed: u64, half_life: u64, level_bips: u64) -> u128 {
        if half_life == 0 {
            return x; // Prevent division by zero
        }
        
        let mut z = x >> (elapsed / half_life);
        
        // Apply linear interpolation for partial half-life
        let partial = (elapsed % half_life) as u128;
        let half_life_u128 = half_life as u128;
        
        if z > 0 && partial > 0 && half_life_u128 > 0 {
            let adjustment = z.saturating_mul(partial).saturating_div(half_life_u128) >> 1;
            z = z.saturating_sub(adjustment);
        }
        
        // Apply level floor
        if level_bips > 0 && level_bips <= 10000 {
            let level_adjustment = (x.saturating_sub(z)).saturating_mul(level_bips as u128).saturating_div(10000);
            z = z.saturating_add(level_adjustment);
        }
        
        z
    }
    
    /// Calculate the current spot price
    /// 
    /// # Formula
    /// 
    /// price = decayed_virtual_input * SCALING_FACTOR / (available_debt + virtual_output)
    pub fn get_current_price(&self, available_debt: u128) -> u128 {
        let elapsed = Self::current_timestamp().saturating_sub(self.pricing.last_update);
        
        let decayed_input = Self::exp_to_level(
            self.pricing.virtual_input_reserves,
            elapsed,
            self.pricing.half_life,
            self.pricing.level_bips
        );
        
        let denominator = available_debt.saturating_add(self.pricing.virtual_output_reserves);
        
        if denominator == 0 {
            return 0; // Prevent division by zero
        }
        
        decayed_input.saturating_mul(SCALING_FACTOR).saturating_div(denominator)
    }
    
    /// Calculate the amount of alkane to mint for a given diesel amount
    /// 
    /// # Formula
    /// 
    /// output = input * (available_debt + virtual_output) / (decayed_virtual_input + input)
    pub fn get_amount_out(&self, input: u128, available_debt: u128) -> u128 {
        if input == 0 {
            return 0;
        }
        
        let elapsed = Self::current_timestamp().saturating_sub(self.pricing.last_update);
        
        let decayed_input = Self::exp_to_level(
            self.pricing.virtual_input_reserves,
            elapsed,
            self.pricing.half_life,
            self.pricing.level_bips
        );
        
        let numerator = input.saturating_mul(available_debt.saturating_add(self.pricing.virtual_output_reserves));
        let denominator = decayed_input.saturating_add(input);
        
        if denominator == 0 {
            return 0; // Prevent division by zero
        }
        
        let result = numerator.saturating_div(denominator);
        
        // Ensure we return at least 1 for non-zero inputs
        if input > 0 && result == 0 {
            return 1;
        }
        
        result
    }
    
    /// Calculate the amount of alkane that can be redeemed from a bond
    /// 
    /// # Formula
    /// 
    /// redeemable = owed * min(elapsed, term) / term - redeemed
    pub fn get_redeem_amount(&self, owed: u128, redeemed: u128, creation: u64) -> u128 {
        let elapsed = Self::current_timestamp().saturating_sub(creation);
        let elapsed_capped = if elapsed > self.term { self.term } else { elapsed };
        
        if self.term == 0 {
            return 0; // Prevent division by zero
        }
        
        let total_redeemable = owed.saturating_mul(elapsed_capped as u128).saturating_div(self.term as u128);
        
        total_redeemable.saturating_sub(redeemed)
    }
    
    /// Purchase a bond
    /// Returns the amount of alkane to be received when the bond fully matures
    pub fn purchase_bond(&mut self, diesel_amount: u128, available_debt: u128) -> u128 {
        let alkane_amount = self.get_amount_out(diesel_amount, available_debt);
        
        if alkane_amount == 0 {
            return 0;
        }
        
        // Update state
        self.pricing.virtual_input_reserves = self.pricing.virtual_input_reserves.saturating_add(diesel_amount);
        self.total_debt = self.total_debt.saturating_add(alkane_amount);
        
        alkane_amount
    }
    
    /// Redeem a bond
    /// Returns the amount of alkane redeemed
    pub fn redeem_bond(&mut self, bond: &mut Bond) -> u128 {
        let redeemable = self.get_redeem_amount(bond.owed, bond.redeemed, bond.creation);
        
        if redeemable == 0 {
            return 0;
        }
        
        // Update state
        bond.redeemed = bond.redeemed.saturating_add(redeemable);
        self.total_debt = self.total_debt.saturating_sub(redeemable);
        
        redeemable
    }
    
    /// Update the pricing parameters
    pub fn update_pricing(
        &mut self,
        new_virtual_input: Option<u128>,
        new_virtual_output: Option<u128>,
        new_half_life: Option<u64>,
        new_level_bips: Option<u64>,
        update_timestamp: bool
    ) {
        if let Some(value) = new_virtual_input {
            self.pricing.virtual_input_reserves = value;
        }
        
        if let Some(value) = new_virtual_output {
            self.pricing.virtual_output_reserves = value;
        }
        
        if let Some(value) = new_half_life {
            self.pricing.half_life = value;
        }
        
        if let Some(value) = new_level_bips {
            self.pricing.level_bips = value;
        }
        
        if update_timestamp {
            self.pricing.last_update = Self::current_timestamp();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exp_to_level() {
        // Test with no decay (elapsed = 0)
        let x = 1000;
        let elapsed = 0;
        let half_life = 3600; // 1 hour
        let level_bips = 5000; // 50%
        
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, x, "No decay should return original value");
        
        // Test with one half-life elapsed
        let elapsed = 3600; // 1 hour
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 500, "One half-life should reduce to 50%");
        
        // Test with two half-lives elapsed
        let elapsed = 7200; // 2 hours
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 250, "Two half-lives should reduce to 25%");
        
        // Test with level_bips = 0 (no floor)
        let level_bips = 0;
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 250, "With no floor, should decay to 25%");
        
        // Test with level_bips = 10000 (100% floor)
        let level_bips = 10000;
        let result = BondCurve::exp_to_level(x, elapsed, half_life, level_bips);
        assert_eq!(result, 1000, "With 100% floor, should remain at original value");
    }
    
    #[test]
    fn test_get_current_price() {
        // Create a bond curve with initial parameters
        let curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (1 hour)
            5000,    // level bips (50%)
            86400    // term (24 hours)
        );
        
        // Test with available debt = 500000
        let available_debt = 500000;
        let price = curve.get_current_price(available_debt);
        
        // Expected price = 1000000 * SCALING_FACTOR / (500000 + 500000) = SCALING_FACTOR
        assert_eq!(price, SCALING_FACTOR, "Initial price should be SCALING_FACTOR");
    }
    
    #[test]
    fn test_get_amount_out() {
        // Create a bond curve with initial parameters
        let curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (1 hour)
            5000,    // level bips (50%)
            86400    // term (24 hours)
        );
        
        // Test with input = 100000, available debt = 500000
        let input = 100000;
        let available_debt = 500000;
        let output = curve.get_amount_out(input, available_debt);
        
        // Expected output = 100000 * (500000 + 500000) / (1000000 + 100000) = 90909
        assert!(output > 90000 && output < 91000, "Output should be approximately 90909");
    }
    
    #[test]
    fn test_get_redeem_amount() {
        // Create a bond curve with initial parameters
        let curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (1 hour)
            5000,    // level bips (50%)
            86400    // term (24 hours)
        );
        
        // Create a bond that was created 24 hours ago (fully matured)
        let owed = 100000;
        let redeemed = 0;
        let creation = BondCurve::current_timestamp() - 86400;
        
        let redeemable = curve.get_redeem_amount(owed, redeemed, creation);
        assert_eq!(redeemable, owed, "Fully matured bond should be fully redeemable");
        
        // Create a bond that was created 12 hours ago (half matured)
        let creation = BondCurve::current_timestamp() - 43200;
        let redeemable = curve.get_redeem_amount(owed, redeemed, creation);
        assert_eq!(redeemable, owed / 2, "Half matured bond should be half redeemable");
        
        // Test with already partially redeemed bond
        let redeemed = 25000;
        let redeemable = curve.get_redeem_amount(owed, redeemed, creation);
        assert_eq!(redeemable, owed / 2 - redeemed, "Should account for already redeemed amount");
    }
    
    #[test]
    fn test_purchase_bond() {
        // Create a bond curve with initial parameters
        let mut curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (1 hour)
            5000,    // level bips (50%)
            86400    // term (24 hours)
        );
        
        // Purchase a bond with 100000 diesel
        let available_debt = 500000;
        let alkane_amount = curve.purchase_bond(100000, available_debt);
        
        // Expected output = 100000 * (500000 + 500000) / (1000000 + 100000) = 90909
        assert!(alkane_amount > 90000 && alkane_amount < 91000, "Output should be approximately 90909");
        
        // Verify state updates
        assert_eq!(curve.pricing.virtual_input_reserves, 1100000, "Virtual input reserves should increase");
        assert_eq!(curve.total_debt, alkane_amount, "Total debt should increase by alkane amount");
    }
    
    #[test]
    fn test_redeem_bond() {
        // Create a bond curve with initial parameters
        let mut curve = BondCurve::new(
            1000000, // virtual input reserves
            500000,  // virtual output reserves
            3600,    // half-life (1 hour)
            5000,    // level bips (50%)
            86400    // term (24 hours)
        );
        
        // Create a bond that was created 12 hours ago (half matured)
        let mut bond = Bond {
            owed: 100000,
            redeemed: 0,
            creation: BondCurve::current_timestamp() - 43200,
        };
        
        // Set initial total debt
        curve.total_debt = 100000;
        
        // Redeem the bond
        let redeemed = curve.redeem_bond(&mut bond);
        
        // Expected redeemed = 100000 / 2 = 50000
        assert_eq!(redeemed, 50000, "Should redeem half of the bond");
        
        // Verify state updates
        assert_eq!(bond.redeemed, 50000, "Bond redeemed amount should increase");
        assert_eq!(curve.total_debt, 50000, "Total debt should decrease by redeemed amount");
    }
}
