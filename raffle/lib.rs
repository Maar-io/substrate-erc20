#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffle {
    
    const ENTRIES_LIMIT: u8 = 5;
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Raffle {
        entries: u8,
    }

    impl Raffle {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { 
                entries: 0,
             }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn participate(&mut self) {
            if self.entries < ENTRIES_LIMIT{
                self.entries += 1;
            }
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn participants(&self) -> u8 {
            self.entries
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            let raffle = Raffle::new();
            assert_eq!(raffle.participants(), 0);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let mut raffle = Raffle::new();
            raffle.participate();
            assert_eq!(raffle.participants(), 1);
        }
    }
}
