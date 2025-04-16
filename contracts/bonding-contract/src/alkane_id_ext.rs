use alkanes_support::id::AlkaneId;

/// Extension trait for AlkaneId to provide additional functionality
pub trait AlkaneIdExt {
    /// Convert the AlkaneId to a u128 value
    /// 
    /// This is primarily used for testing purposes where we need to use
    /// the AlkaneId as a key or identifier in a simple way.
    fn into_u128(&self) -> u128;
}

impl AlkaneIdExt for AlkaneId {
    fn into_u128(&self) -> u128 {
        // Use the block field as the u128 representation
        // This is a simplification for testing purposes
        self.block
    }
}
