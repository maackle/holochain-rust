use cas::content::{Address, AddressableContent, Content};
use entry::{test_entry, Entry};
use error::HolochainError;
use json::{FromJson, RoundTripJson, ToJson};
use keys::test_keys;
use multihash::Hash;
use serde_json;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Meta represents an extended form of EAV (entity-attribute-value) data
/// E = the entry key for hash table lookups
/// A = the name of the meta attribute
/// V = the value of the meta attribute
/// txn = a unique (local to the source) monotonically increasing number that can be used for
///       crdt/ordering
///       @see https://papers.radixdlt.com/tempo/#logical-clocks
/// source = the agent making the meta assertion
/// signature = the asserting agent's signature of the meta assertion
pub struct EntryMeta {
    entry_address: Address,
    attribute: String,
    value: String,
    // @TODO implement local transaction ordering
    // @see https://github.com/holochain/holochain-rust/issues/138
    // txn: String,
    source: String,
    // @TODO implement meta data signing
    // @see https://github.com/holochain/holochain-rust/issues/139
    // signature: String,
}

impl Ord for EntryMeta {
    fn cmp(&self, other: &EntryMeta) -> Ordering {
        // we want to sort by entry hash, then attribute name, then attribute value
        match self.entry_address.cmp(&other.entry_address) {
            Ordering::Equal => match self.attribute.cmp(&other.attribute) {
                Ordering::Equal => self.value.cmp(&other.value),
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
            },
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
        }
    }
}

impl PartialOrd for EntryMeta {
    fn partial_cmp(&self, other: &EntryMeta) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl EntryMeta {
    /// Builds a new Meta from EAV and agent keys, where E is an existing Entry
    /// @TODO need a `from()` to build a local meta from incoming network messages
    /// @see https://github.com/holochain/holochain-rust/issues/140
    pub fn new(node_id: &str, address: &Address, attribute: &str, value: &str) -> EntryMeta {
        EntryMeta {
            entry_address: address.clone(),
            attribute: attribute.into(),
            value: value.into(),
            source: node_id.to_string(),
        }
    }

    /// getter for entry
    pub fn entry_address(&self) -> &Address {
        &self.entry_address
    }

    /// getter for attribute clone
    pub fn attribute(&self) -> String {
        self.attribute.clone()
    }

    /// getter for value clone
    pub fn value(&self) -> String {
        self.value.clone()
    }

    /// getter for source clone
    pub fn source(&self) -> String {
        self.source.clone()
    }

    pub fn make_address(address: &Address, attribute: &str) -> Address {
        let pieces: [String; 2] = [address.clone().to_string(), attribute.to_string()];
        let string_to_address = pieces.concat();

        // @TODO the hashing algo should not be hardcoded
        // @see https://github.com/holochain/holochain-rust/issues/104
        Address::encode_from_str(&string_to_address, Hash::SHA2256)
    }
}

impl ToJson for EntryMeta {
    fn to_json(&self) -> Result<String, HolochainError> {
        Ok(serde_json::to_string(&self)?)
    }
}

impl FromJson for EntryMeta {
    /// @TODO accept canonical JSON
    /// @see https://github.com/holochain/holochain-rust/issues/75
    fn from_json(s: &str) -> Result<Self, HolochainError> {
        Ok(serde_json::from_str(s)?)
    }
}

impl RoundTripJson for EntryMeta {}

impl AddressableContent for EntryMeta {
    fn address(&self) -> Address {
        EntryMeta::make_address(&self.entry_address, &self.attribute)
    }

    fn content(&self) -> Content {
        self.to_json().expect("could not Jsonify EntryMeta Content")
    }

    fn from_content(content: &Content) -> Self {
        EntryMeta::from_json(content).expect("could not parse JSON as EntryMeta Content")
    }
}

/// dummy test attribute name
pub fn test_attribute() -> String {
    "meta-attribute".into()
}

/// dummy test attribute name, same as test_attribute()
pub fn test_attribute_a() -> String {
    test_attribute()
}

/// dummy test attribute name, differs from test_attribute()
pub fn test_attribute_b() -> String {
    "another-attribute".into()
}

/// dummy test attribute value
pub fn test_value() -> String {
    "meta value".into()
}

/// dummy test attribute value, same as test_value()
pub fn test_value_a() -> String {
    test_value()
}

/// dummy test attribute value, differs from test_value()
pub fn test_value_b() -> String {
    "another value".into()
}

pub fn test_meta_for(entry: &Entry, attribute: &str, value: &str) -> EntryMeta {
    EntryMeta::new(&test_keys().node_id(), &entry.address(), attribute, value)
}

/// returns dummy meta for testing
pub fn test_meta() -> EntryMeta {
    EntryMeta::new(
        &test_keys().node_id(),
        &test_entry().address(),
        &test_attribute(),
        &test_value(),
    )
}

/// dummy meta, same as test_meta()
pub fn test_meta_a() -> EntryMeta {
    test_meta()
}

/// returns dummy meta for testing against the same entry as test_meta_a
pub fn test_meta_b() -> EntryMeta {
    EntryMeta::new(
        &test_keys().node_id(),
        &test_entry().address(),
        &test_attribute_b(),
        &test_value_b(),
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use cas::content::{Address, AddressableContent};
    use entry::test_entry;
    use json::{FromJson, ToJson};
    use keys::test_keys;
    use std::cmp::Ordering;

    #[test]
    /// smoke test EntryMeta::new()
    fn new() {
        test_meta();
    }

    #[test]
    // test meta.entry_address()
    fn entry_address_test() {
        assert_eq!(test_meta().entry_address(), &test_entry().address());
    }

    /// test meta.attribute()
    #[test]
    fn attribute() {
        assert_eq!(test_meta().attribute(), test_attribute());
    }

    #[test]
    /// test meta.value()
    fn value() {
        assert_eq!(test_meta().value(), test_value());
    }

    #[test]
    /// test meta.source()
    fn source() {
        assert_eq!(test_meta().source(), test_keys().node_id());
    }

    #[test]
    /// test that we can sort metas with cmp
    fn cmp() {
        // basic ordering
        let m_1ax = EntryMeta::new(
            &test_keys().node_id(),
            &Address::from("1".to_string()),
            "a",
            "x",
        );
        let m_1ay = EntryMeta::new(
            &test_keys().node_id(),
            &Address::from("1".to_string()),
            "a",
            "y",
        );
        let m_1bx = EntryMeta::new(
            &test_keys().node_id(),
            &Address::from("1".to_string()),
            "b",
            "x",
        );
        let m_2ax = EntryMeta::new(
            &test_keys().node_id(),
            &Address::from("2".to_string()),
            "a",
            "x",
        );

        // sort by entry key
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_2ax));
        assert_eq!(Ordering::Equal, m_1ax.cmp(&m_1ax));
        assert_eq!(Ordering::Greater, m_2ax.cmp(&m_1ax));
        assert_eq!(Ordering::Less, m_1ay.cmp(&m_2ax));

        // entry key with operators
        assert!(m_1ax < m_2ax);
        assert!(m_2ax > m_1ax);
        assert!(m_1ay < m_2ax);

        // sort by attribute key
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_1bx));
        assert_eq!(Ordering::Greater, m_1bx.cmp(&m_1ax));

        // attribute key with operators
        assert!(m_1ax < m_1bx);
        assert!(m_1bx > m_1ax);

        // sort by attribute value
        assert_eq!(Ordering::Less, m_1ax.cmp(&m_1ay));
        assert_eq!(Ordering::Greater, m_1ay.cmp(&m_1ax));

        // attribute value with operators
        assert!(m_1ax < m_1ay);
        assert!(m_1ay > m_1ax);
    }

    #[test]
    /// test the RoundTripJson implementation
    fn test_json_round_trip() {
        let meta = test_meta();
        let expected = "{\"entry_address\":\"QmbXSE38SN3SuJDmHKSSw5qWWegvU7oTxrLDRavWjyxMrT\",\"attribute\":\"meta-attribute\",\"value\":\"meta value\",\"source\":\"test node id\"}";

        assert_eq!(expected.to_string(), meta.to_json().unwrap());
        assert_eq!(meta, EntryMeta::from_json(&expected).unwrap());
        assert_eq!(
            meta,
            EntryMeta::from_json(&meta.to_json().unwrap()).unwrap()
        );
    }
}
