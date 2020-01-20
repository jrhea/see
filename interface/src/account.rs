use bonsai::subtree_index_to_general;
use oof::Oof;
use proof::Provable;

const BALANCE_IDX: u128 = 5;
const NONCE_IDX: u128 = 6;
#[allow(dead_code)]
const PADDING_IDX: u128 = 7;
const PUBKEY_1_IDX: u128 = 8;
const PUBKEY_2_IDX: u128 = 9;

#[derive(Clone)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
    pub pubkey: [u8; 48],
}

pub struct RefAccount {
    idx: u128,
    backend: *mut Oof,
}

impl RefAccount {
    pub fn balance(&self) -> u64 {
        let bytes = Oof::get(unsafe { &*self.backend }, &self.balance_key()).unwrap();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[0..8]);
        u64::from_le_bytes(buf)
    }

    pub fn set_balance(&mut self, balance: u64) {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&balance.to_le_bytes());
        Oof::set(unsafe { &mut *self.backend }, self.balance_key(), buf);
    }

    pub fn nonce(&self) -> u64 {
        let bytes = Oof::get(unsafe { &*self.backend }, &self.nonce_key()).unwrap();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[0..8]);
        u64::from_le_bytes(buf)
    }

    pub fn inc_nonce(&mut self) {
        let nonce = self.nonce() + 1;
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&nonce.to_le_bytes());
        Oof::set(unsafe { &mut *self.backend }, self.nonce_key(), buf);
    }

    pub fn pubkey(&self) -> [u8; 48] {
        let bytes_0 = Oof::get(
            unsafe { &*self.backend },
            &subtree_index_to_general(self.idx, PUBKEY_1_IDX),
        )
        .unwrap();
        let bytes_1 = Oof::get(
            unsafe { &*self.backend },
            &subtree_index_to_general(self.idx, PUBKEY_2_IDX),
        )
        .unwrap();

        let mut buf = [0u8; 48];
        buf[0..32].copy_from_slice(&bytes_0[0..32]);
        buf[32..48].copy_from_slice(&bytes_1[0..16]);
        buf
    }

    const fn balance_key(&self) -> u128 {
        subtree_index_to_general(self.idx, BALANCE_IDX)
    }

    const fn nonce_key(&self) -> u128 {
        subtree_index_to_general(self.idx, NONCE_IDX)
    }
}

impl Provable for RefAccount {
    fn new(idx: u128, backend: *mut Oof) -> Self {
        Self { idx, backend }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::collections::BTreeMap;
    use arrayref::array_ref;
    use oof::hash::hash;

    fn make_value(val: &[u8]) -> [u8; 32] {
        let mut buf = [0; 32];
        buf[0..val.len()].copy_from_slice(val);
        buf
    }

    impl Account {
        fn to_map(self) -> BTreeMap<u128, [u8; 32]> {
            let mut map = BTreeMap::default();

            map.insert(BALANCE_IDX, make_value(&self.balance.to_le_bytes()));
            map.insert(NONCE_IDX, make_value(&self.nonce.to_le_bytes()));
            map.insert(PUBKEY_1_IDX, make_value(&self.pubkey[0..32]));
            map.insert(PUBKEY_2_IDX, make_value(&self.pubkey[32..48]));
            map.insert(PADDING_IDX, [0; 32]);

            map
        }
    }

    #[test]
    fn test() {
        let a = Account {
            balance: 1,
            nonce: 2,
            pubkey: [1; 48],
        };

        let mut oof = Oof::from_map(a.clone().to_map());
        let ptr = RefAccount::new(1, &mut oof as *mut Oof);

        assert_eq!(ptr.balance(), a.balance);
        assert_eq!(ptr.nonce(), a.nonce);
        assert_eq!(ptr.pubkey()[..], a.pubkey[..]);

        let four = hash(array_ref![a.pubkey, 0, 32], &make_value(&a.pubkey[32..48]));
        let two = hash(&four, &make_value(&a.balance.to_le_bytes()));
        let three = hash(&make_value(&a.nonce.to_le_bytes()), &[0; 32]);
        let one = hash(&two, &three);

        assert_eq!(oof.root(), Ok(&one));
    }
}
