use bam_api::header::{FromBamHeader, ToBamHeader};
use bitcoin_support::BitcoinQuantity;
use ethereum_support::{Erc20Quantity, EtherQuantity};
use http_api::asset::{FromHttpAsset, ToHttpAsset};
use std::fmt::Debug;

pub trait Asset:
    Clone
    + Debug
    + Send
    + Sync
    + 'static
    + PartialEq
    + FromHttpAsset
    + ToHttpAsset
    + FromBamHeader
    + ToBamHeader
{
}

impl Asset for BitcoinQuantity {}
impl Asset for EtherQuantity {}
impl Asset for Erc20Quantity {}
