#![cfg(test)]

use crate::format_id;
use soroban_sdk::{Env};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_format_id_starts_with_prefix(n in 0u64..u64::MAX) {
        let env = Env::default();
        let prefix = "refund_";
        let id = format_id(&env, prefix, n);
        
        let mut arr = [0u8; 64];
        let len = id.len() as usize;
        id.copy_into_slice(&mut arr[..len]);
        let id_str = core::str::from_utf8(&arr[..len]).unwrap();
        
        assert!(id_str.starts_with(prefix));
    }

    #[test]
    fn test_format_id_uniqueness(n1 in 0u64..u64::MAX, n2 in 0u64..u64::MAX) {
        prop_assume!(n1 != n2);
        let env = Env::default();
        let prefix = "id_";
        let id1 = format_id(&env, prefix, n1);
        let id2 = format_id(&env, prefix, n2);
        
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_format_id_round_trip(n in 1u64..u64::MAX) {
        let env = Env::default();
        let prefix = "dispute_";
        let id = format_id(&env, prefix, n);
        
        let mut arr = [0u8; 64];
        let len = id.len() as usize;
        id.copy_into_slice(&mut arr[..len]);
        let id_str = core::str::from_utf8(&arr[..len]).unwrap();
        
        // Extract the number part
        let num_part = &id_str[prefix.len()..];
        let parsed_n: u64 = num_part.parse().unwrap();
        
        assert_eq!(n, parsed_n);
    }
}
