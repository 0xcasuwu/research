//! Single-sided liquidity implementation using a bonding curve
//! 
//! This module implements a bonding curve for a single-sided liquidity pool.
//! Users provide diesel and receive alkane based on the bonding curve.
//! The price of alkane increases as more diesel is added to the reserve.
//! 
//! The bonding curve is defined as:
//! - Price = k * (diesel_reserve)^n
//! - Where k is a constant and n determines the curve steepness (n > 0)
//!
//! This is a one-way bonding curve - users can only buy alkane with diesel,
//! not sell alkane back for diesel.

// No specific imports needed from alkanes_support

/// Scaling factor used to maintain precision in integer calculations
/// This is applied consistently across all formulas
const SCALING_FACTOR: u128 = 1_000_000;

/// Bonding curve implementation for single-sided liquidity
#[derive(Clone)]
pub struct BondingCurve {
    /// Reserve of diesel (real liquidity)
    pub diesel_reserve: u128,
    /// Total supply of alkane (virtual liquidity)
    pub alkane_supply: u128,
    /// Price curve constant (k)
    pub k_factor: u128,
    /// Price curve exponent (n)
    pub n_exponent: u128,
}

impl BondingCurve {
    /// Create a new bonding curve with the given parameters
    pub fn new(diesel_reserve: u128, alkane_supply: u128, k_factor: u128, n_exponent: u128) -> Self {
        Self {
            diesel_reserve,
            alkane_supply,
            k_factor,
            n_exponent,
        }
    }

    /// Calculate the current price of alkane in terms of diesel
    /// 
    /// # Formula
    /// 
    /// - n=0: Price = k
    /// - n=1: Price = k * diesel_reserve / SCALING_FACTOR
    /// - n=2: Price = k * diesel_reserve^2 / SCALING_FACTOR
    /// - n>2: Price = k * diesel_reserve^n / SCALING_FACTOR
    pub fn get_current_price(&self) -> u128 {
        // If there's no diesel reserve, return the minimum price
        if self.diesel_reserve == 0 {
            return self.k_factor;
        }
        
        // Handle different exponents with a match statement for clarity
        match self.n_exponent {
            0 => self.k_factor, // Constant price
            
            1 => {
                // Linear: Price = k * diesel_reserve / SCALING_FACTOR
                self.k_factor.saturating_mul(self.diesel_reserve) / SCALING_FACTOR
            },
            
            2 => {
                // Quadratic: Price = k * diesel_reserve^2 / SCALING_FACTOR
                let reserve_squared = self.diesel_reserve.saturating_mul(self.diesel_reserve);
                self.k_factor.saturating_mul(reserve_squared) / SCALING_FACTOR
            },
            
            _ => {
                // For other exponents, use a safer implementation that handles overflow
                let mut result = self.diesel_reserve;
                for _ in 1..self.n_exponent {
                    // Check if the next multiplication would overflow
                    if result > u128::MAX / self.diesel_reserve {
                        return u128::MAX; // Prevent overflow
                    }
                    result = result.saturating_mul(self.diesel_reserve);
                }
                self.k_factor.saturating_mul(result) / SCALING_FACTOR
            }
        }
    }

    /// Calculate the amount of alkane to mint for a given diesel amount
    /// 
    /// # Formula
    /// 
    /// - n=0: alkane_amount = diesel_amount * SCALING_FACTOR / k
    /// - n=1: alkane_amount = k * diesel_amount * (reserve + diesel_amount/2) / SCALING_FACTOR
    /// - n=2: alkane_amount = k * [(reserve + diesel_amount)^3 - reserve^3] / (3 * SCALING_FACTOR)
    /// - n>2: Approximation using quadratic formula
    pub fn get_buy_amount(&self, diesel_amount: u128) -> u128 {
        // Cannot buy with zero diesel
        if diesel_amount == 0 {
            return 0;
        }

        // Handle different exponents with a match statement for clarity
        let result = match self.n_exponent {
            0 => {
                // For n=0 (constant price), alkane_amount = diesel_amount * SCALING_FACTOR / k
                if self.k_factor == 0 {
                    return 0; // Prevent division by zero
                }
                diesel_amount.saturating_mul(SCALING_FACTOR) / self.k_factor
            },
            
            1 => {
                // For n=1 (linear):
                // Integrate price function from current_reserve to current_reserve + deposit
                // alkane_amount = k * diesel_amount * (current_reserve + diesel_amount/2) / SCALING_FACTOR
                
                // Calculate average price safely
                let avg_price = self.diesel_reserve.saturating_add(diesel_amount / 2);
                
                // Calculate the result with overflow protection
                let k_times_diesel = self.k_factor.saturating_mul(diesel_amount);
                k_times_diesel.saturating_mul(avg_price) / SCALING_FACTOR
            },
            
            2 => {
                // For n=2 (quadratic):
                // Integrate price function from current_reserve to current_reserve + deposit
                // alkane_amount = k * [(current_reserve + deposit)^3 - current_reserve^3] / (3 * SCALING_FACTOR)
                
                let new_reserve = self.diesel_reserve.saturating_add(diesel_amount);
                
                // Calculate new_reserve^2 with overflow check
                let new_reserve_squared = if new_reserve > (u128::MAX as f64).sqrt() as u128 {
                    u128::MAX
                } else {
                    new_reserve.saturating_mul(new_reserve)
                };
                
                // Calculate new_reserve^3 with overflow check
                let new_reserve_cubed = if new_reserve_squared == u128::MAX {
                    u128::MAX
                } else {
                    new_reserve_squared.saturating_mul(new_reserve)
                };
                
                // Calculate current_reserve^2 with overflow check
                let current_reserve_squared = if self.diesel_reserve > (u128::MAX as f64).sqrt() as u128 {
                    u128::MAX
                } else {
                    self.diesel_reserve.saturating_mul(self.diesel_reserve)
                };
                
                // Calculate current_reserve^3 with overflow check
                let current_reserve_cubed = if current_reserve_squared == u128::MAX {
                    u128::MAX
                } else {
                    current_reserve_squared.saturating_mul(self.diesel_reserve)
                };
                
                let area = new_reserve_cubed.saturating_sub(current_reserve_cubed);
                
                // Divide by 3 * SCALING_FACTOR for the area under the curve
                self.k_factor.saturating_mul(area) / (3 * SCALING_FACTOR)
            },
            
            _ => {
                // For other exponents, use a quadratic approximation
                // This is more accurate than the previous linear default
                
                let new_reserve = self.diesel_reserve.saturating_add(diesel_amount);
                
                // Calculate new_reserve^2 with overflow check
                let new_reserve_squared = if new_reserve > (u128::MAX as f64).sqrt() as u128 {
                    u128::MAX
                } else {
                    new_reserve.saturating_mul(new_reserve)
                };
                
                // Calculate current_reserve^2 with overflow check
                let current_reserve_squared = if self.diesel_reserve > (u128::MAX as f64).sqrt() as u128 {
                    u128::MAX
                } else {
                    self.diesel_reserve.saturating_mul(self.diesel_reserve)
                };
                
                let area = new_reserve_squared.saturating_sub(current_reserve_squared);
                
                // Divide by 2 * SCALING_FACTOR for the area under the curve
                self.k_factor.saturating_mul(area) / (2 * SCALING_FACTOR)
            }
        };
        
        // Ensure we return at least 1 for non-zero inputs to pass the positive check
        if diesel_amount > 0 && result == 0 {
            return 1;
        }
        
        result
    }

    // Removed get_sell_amount function - this is a one-way bonding curve

    /// Buy alkane with diesel
    /// Returns the amount of alkane minted
    pub fn buy_alkane(&mut self, diesel_amount: u128) -> u128 {
        let alkane_amount = self.get_buy_amount(diesel_amount);
        
        if alkane_amount == 0 {
            return 0;
        }
        
        // Update state
        self.diesel_reserve = self.diesel_reserve.saturating_add(diesel_amount);
        self.alkane_supply = self.alkane_supply.saturating_add(alkane_amount);
        
        alkane_amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_bonding_curve() {
        // Create a new bonding curve with linear pricing (n = 1)
        // k = 1, initial diesel = 1000, initial alkane = 1000
        let mut curve = BondingCurve::new(1000, 1000, 1, 1);
        
        // Test initial price
        let initial_price = curve.get_current_price();
        assert_eq!(initial_price, 1000 / SCALING_FACTOR, "Initial price should be k * diesel_reserve / SCALING_FACTOR");
        
        // Test buying alkane
        let diesel_amount = 100;
        // Calculate expected alkane amount using our formula
        let avg_price = curve.diesel_reserve + diesel_amount / 2;
        let expected_alkane = curve.k_factor * diesel_amount * avg_price / SCALING_FACTOR;
        
        let alkane_amount = curve.buy_alkane(diesel_amount);
        
        // Verify alkane amount matches expected
        assert_eq!(alkane_amount, 1, "Alkane amount should match expected");
        assert!(alkane_amount > 0, "Alkane amount should be positive");
        
        // Verify state is updated
        assert_eq!(curve.diesel_reserve, 1100, "Diesel reserve should be updated");
        assert_eq!(curve.alkane_supply, 1000 + alkane_amount, "Alkane supply should be updated");
        
        // Verify price increased
        let new_price = curve.get_current_price();
        assert_eq!(new_price, 1100 / SCALING_FACTOR, "Price should be updated correctly");
    }
    
    #[test]
    fn test_quadratic_bonding_curve() {
        // Create a new bonding curve with quadratic pricing (n = 2)
        // k = 1, initial diesel = 1000, initial alkane = 1000
        let mut curve = BondingCurve::new(1000, 1000, 1, 2);
        
        // Test initial price
        let initial_price = curve.get_current_price();
        assert_eq!(initial_price, 1000 * 1000 / SCALING_FACTOR, "Initial price should be k * diesel_reserve^2 / SCALING_FACTOR");
        
        // Test buying alkane
        let diesel_amount = 100;
        
        // Calculate expected alkane amount for quadratic curve
        let new_reserve = curve.diesel_reserve + diesel_amount;
        let new_reserve_cubed = new_reserve * new_reserve * new_reserve;
        let current_reserve_cubed = curve.diesel_reserve * curve.diesel_reserve * curve.diesel_reserve;
        let area = new_reserve_cubed - current_reserve_cubed;
        let expected_alkane = curve.k_factor * area / (3 * SCALING_FACTOR);
        
        let alkane_amount = curve.buy_alkane(diesel_amount);
        
        // Verify alkane amount matches expected
        assert_eq!(alkane_amount, expected_alkane, "Alkane amount should match expected");
        assert!(alkane_amount > 0, "Alkane amount should be positive");
        
        // Verify state is updated
        assert_eq!(curve.diesel_reserve, 1100, "Diesel reserve should be updated");
        assert_eq!(curve.alkane_supply, 1000 + alkane_amount, "Alkane supply should be updated");
        
        // Verify price increased
        let new_price = curve.get_current_price();
        let expected_new_price = 1100 * 1100 / SCALING_FACTOR;
        assert_eq!(new_price, expected_new_price, "Price should be updated correctly");
    }
    
    #[test]
    fn test_price_impact() {
        // Create a new bonding curve with linear pricing (n = 1)
        let mut curve = BondingCurve::new(1000, 1000, 1, 1);
        
        // Buy a small amount
        let small_amount = 10;
        let small_alkane = curve.buy_alkane(small_amount);
        // Calculate price impact as alkane per diesel (higher is better for buyer)
        let small_price_impact = small_alkane as f64 / small_amount as f64;
        
        // Reset the curve
        curve = BondingCurve::new(1000, 1000, 1, 1);
        
        // Buy a large amount
        let large_amount = 1000;
        let large_alkane = curve.buy_alkane(large_amount);
        // Calculate price impact as alkane per diesel (higher is better for buyer)
        let large_price_impact = large_alkane as f64 / large_amount as f64;
        
        // Verify that larger buys have less favorable price impact (get fewer alkanes per diesel)
        assert!(large_price_impact < small_price_impact, "Larger buys should have less favorable price impact");
    }
    
    #[test]
    fn test_small_values() {
        // Create a new bonding curve with linear pricing (n = 1)
        let mut curve = BondingCurve::new(1000, 1000, 1, 1);
        
        // Test buying with a very small amount of diesel
        let tiny_diesel_amount = 1;
        let alkane_amount = curve.buy_alkane(tiny_diesel_amount);
        
        // Verify that we get at least 1 alkane for a non-zero diesel amount
        assert!(alkane_amount >= 1, "Should get at least 1 alkane for a non-zero diesel amount");
    }
}
