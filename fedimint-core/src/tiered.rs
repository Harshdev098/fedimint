use std::collections::BTreeMap;

use fedimint_core::Amount;
use serde::{Deserialize, Serialize};

use crate::encoding::{Decodable, DecodeError, Encodable};
use crate::module::registry::ModuleDecoderRegistry;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct InvalidAmountTierError(pub Amount);

impl std::fmt::Display for InvalidAmountTierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Amount tier unknown to mint: {}", self.0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Tiered<T>(BTreeMap<Amount, T>);

impl<T> Default for Tiered<T> {
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

impl<T> Tiered<T> {
    /// Returns the highest tier amount
    pub fn max_tier(&self) -> &Amount {
        self.0.keys().max().expect("has tiers")
    }

    pub fn structural_eq<O>(&self, other: &Tiered<O>) -> bool {
        self.0.keys().eq(other.0.keys())
    }

    /// Returns a reference to the key of the specified tier
    pub fn tier(&self, amount: &Amount) -> Result<&T, InvalidAmountTierError> {
        self.0.get(amount).ok_or(InvalidAmountTierError(*amount))
    }

    pub fn count_tiers(&self) -> usize {
        self.0.len()
    }

    pub fn tiers(&self) -> impl DoubleEndedIterator<Item = &Amount> {
        self.0.keys()
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = &T> {
        self.0.values()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Amount, &T)> {
        self.0.iter().map(|(amt, key)| (*amt, key))
    }

    pub fn get(&self, amt: Amount) -> Option<&T> {
        self.0.get(&amt)
    }

    pub fn get_mut(&mut self, amt: Amount) -> Option<&mut T> {
        self.0.get_mut(&amt)
    }

    pub fn insert(&mut self, amt: Amount, v: T) -> Option<T> {
        self.0.insert(amt, v)
    }

    pub fn get_mut_or_default(&mut self, amt: Amount) -> &mut T
    where
        T: Default,
    {
        self.0.entry(amt).or_default()
    }

    pub fn entry(&mut self, amt: Amount) -> std::collections::btree_map::Entry<'_, Amount, T>
    where
        T: Default,
    {
        self.0.entry(amt)
    }

    pub fn as_map(&self) -> &BTreeMap<Amount, T> {
        &self.0
    }
}

impl Tiered<()> {
    /// Generates denominations of a given base up to and including `max`
    pub fn gen_denominations(denomination_base: u16, max: Amount) -> Self {
        let mut amounts = vec![];

        let mut denomination = Amount::from_msats(1);
        while denomination <= max {
            amounts.push((denomination, ()));
            denomination = denomination * denomination_base.into();
        }

        amounts.into_iter().collect()
    }
}

impl<T> FromIterator<(Amount, T)> for Tiered<T> {
    fn from_iter<I: IntoIterator<Item = (Amount, T)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for Tiered<T> {
    type Item = (Amount, T);
    type IntoIter = std::collections::btree_map::IntoIter<Amount, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<C> Encodable for Tiered<C>
where
    C: Encodable,
{
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        self.0.consensus_encode(writer)?;
        Ok(())
    }
}

impl<C> Decodable for Tiered<C>
where
    C: Decodable,
{
    fn consensus_decode_partial<D: std::io::Read>(
        d: &mut D,
        modules: &ModuleDecoderRegistry,
    ) -> Result<Self, DecodeError> {
        Ok(Self(BTreeMap::consensus_decode_partial(d, modules)?))
    }
}

#[cfg(test)]
mod tests;
