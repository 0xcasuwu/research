pub mod balance_sheet;
pub mod balance_sheet_ext;
pub mod byte_utils;
pub mod constants;
pub mod network;
pub mod proto;
pub mod protostone;
pub mod rune_transfer;
pub mod utils;

// Re-export commonly used functions from balance_sheet_ext
pub use balance_sheet_ext::{load_sheet, clear_balances, PersistentRecord, Mintable, MintableDebit, OutgoingRunes};

use anyhow;
use bitcoin::hashes::Hash;
use bitcoin::{OutPoint, Txid};

impl TryInto<OutPoint> for proto::protorune::Outpoint {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<OutPoint, Self::Error> {
        Ok(OutPoint {
            txid: Txid::from_byte_array(<&[u8] as TryInto<[u8; 32]>>::try_into(&self.txid)?),
            vout: self.vout.into(),
        })
    }
}
