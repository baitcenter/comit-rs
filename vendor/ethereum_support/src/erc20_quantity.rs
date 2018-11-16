use bigdecimal::BigDecimal;
use num::{
    bigint::{BigInt, Sign},
    ToPrimitive,
};
use regex::Regex;
use std::{f64, fmt, mem};
use web3::types::Address;
use U256;

const U64SIZE: usize = mem::size_of::<u64>();

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Erc20Quantity {
    name: String,
    decimals: u16,
    address: Address,
    amount: U256,
}

impl Erc20Quantity {
    pub fn with_wei(name: String, decimals: u16, address: Address, wei: U256) -> Self {
        Erc20Quantity {
            name,
            decimals,
            address,
            amount: wei,
        }
    }

    pub fn to_full_token(&self) -> f64 {
        self.to_bigdec().to_f64().unwrap()
    }

    pub fn wei(&self) -> U256 {
        self.amount
    }

    fn to_bigdec(&self) -> BigDecimal {
        let mut bytes = [0u8; U64SIZE * 4];
        self.amount.to_little_endian(&mut bytes);

        let bigint = BigInt::from_bytes_le(Sign::Plus, &bytes);

        BigDecimal::new(bigint, self.decimals.into())
    }
}

lazy_static! {
    static ref TRAILING_ZEROS: Regex = Regex::new(r"\.?0*$").unwrap();
}

impl fmt::Display for Erc20Quantity {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // At time of writing BigDecimal always puts . and pads zeroes
        // up to the precision in f, so TRAILING_ZEROS does the right
        // thing in all cases.
        let fmt_dec = format!("{}", self.to_bigdec());
        let removed_trailing_zeros = TRAILING_ZEROS.replace(fmt_dec.as_str(), "");
        write!(f, "{} {}", removed_trailing_zeros, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erc20_quantity::Erc20Quantity;
    use spectral::prelude::*;
    use std::str::FromStr;
    use web3::types::Address;

    #[test]
    fn create_token_with_16_dec_from_wei() {
        let address = Address::from_str("B97048628DB6B661D4C2aA833e95Dbe1A905B280").unwrap();
        let wei = U256::from(10);
        let erc20quantity = Erc20Quantity::with_wei(String::from("PAY"), 18, address, wei);

        assert_that(&erc20quantity.wei()).is_equal_to(&wei);
    }

    #[test]
    fn create_token_with_18_dec_from_wei_to_full_token() {
        let address = Address::from_str("B97048628DB6B661D4C2aA833e95Dbe1A905B280").unwrap();
        let wei = U256::from(1_000_000_000_000_000_000u64);
        let full_token = 1.0;
        let erc20quantity = Erc20Quantity::with_wei(String::from("PAY"), 18, address, wei);

        assert_that(&erc20quantity.wei()).is_equal_to(&wei);
        assert_that(&erc20quantity.to_full_token()).is_equal_to(&full_token);
    }

    #[test]
    fn create_token_with_16_dec_from_wei_to_full_token() {
        let address = Address::from_str("B97048628DB6B661D4C2aA833e95Dbe1A905B280").unwrap();
        let wei = U256::from(1_000_000_000_000_000_000u64);
        let full_token = 100.0;
        let erc20quantity = Erc20Quantity::with_wei(String::from("PAY"), 16, address, wei);

        assert_that(&erc20quantity.wei()).is_equal_to(&wei);
        assert_that(&erc20quantity.to_full_token()).is_equal_to(&full_token);
    }

    #[test]
    fn given_an_erc20quantity_will_serialize() {
        let address = Address::from_str("B97048628DB6B661D4C2aA833e95Dbe1A905B280").unwrap();
        let wei = U256::from(1_000_000_000_000_000_000u64);
        let erc20quantity = Erc20Quantity::with_wei(String::from("PAY"), 16, address, wei);

        let serialized = serde_json::to_string(&erc20quantity).unwrap();
        assert_eq!(serialized, r#"{"name":"PAY","decimals":16,"address":"0xb97048628db6b661d4c2aa833e95dbe1a905b280","amount":"0xde0b6b3a7640000"}"#)
    }

    #[test]
    fn given_a_deserialized_erc20quantity_will_deserialize() {
        let serialized = r#"{"name":"PAY","decimals":16,"address":"0xb97048628db6b661d4c2aa833e95dbe1a905b280","amount":"0xde0b6b3a7640000"}"#;

        let deserialized: Erc20Quantity = serde_json::from_str(serialized).unwrap();

        assert_that(&deserialized).is_equal_to(Erc20Quantity::with_wei(
            String::from("PAY"),
            16,
            Address::from_str("B97048628DB6B661D4C2aA833e95Dbe1A905B280").unwrap(),
            U256::from(1_000_000_000_000_000_000u64),
        ));
    }
}
