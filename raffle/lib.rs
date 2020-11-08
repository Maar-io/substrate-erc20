#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffle {


    //A user can send in anywhere between 0.01 and 0.1 tokens.
    //const DEPOSIT_MIN: u128 = 1_000_000_000;
    //const DEPOSIT_MAX: u128 = 10_000_000_000;

    // countdown only starts once there are at least RAFFLE_TRIGGER players in the pool
    const RAFFLE_TRIGGER: u32 = 5; 

    // The collected money from the pot is 
    // automatically sent to a pre-defined address when the second winner is drawn.
    //const SEND_POT_ADDRESS: AccountId = 0xfff;

    /// The Raffle error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    /// The Raffle result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive()]
    pub struct Raffle {
        owner: AccountId,
        total_balance: Balance,
        draw_allowed: bool,
        participants: u16,
        participant_list: Vec<AccountId>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct NewParticipant {
        #[ink(topic)]
        participant: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    impl Raffle {
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            let instance = Self { 
                owner,
                total_balance: 0 as Balance,
                draw_allowed: false,
                participants: 0,
                participant_list: Vec::new(),
             };
             instance
        }

        /// A message that can be called on instantiated contracts.
        /// This one accepts ne participant
        /// If amount is not within limits, it is rejected
        #[ink(message)]
            //let participant = self.env().caller();
        pub fn participate(&mut self, participant: AccountId, value: u128) -> Result<()>{
            
            //let participant = self.env().caller();
            let dbg_msg = format!( "new participant {:#?}", participant );
            ink_env::debug_println( &dbg_msg );

            self.participant_list.push(participant);
            self.env().emit_event(NewParticipant {
                participant: Some(participant),
                value,
            });
            if self.participant_list.len() as u32 >= RAFFLE_TRIGGER{
                self.draw_allowed = true;
            }
            Ok(())
        }

        /// Check if account already paid... test only
        #[ink(message)]
        pub fn is_participating(&self, account: AccountId ) -> bool {
            for a in self.participant_list.iter(){
                if a == &account{
                    return true
                }
            }
            false
        }
    }


    #[cfg(test)]
    mod tests {
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let raffle = Raffle::new(accounts.alice);
            assert_eq!(raffle.participants, 0);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn it_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);
            assert_eq!(raffle.participate(accounts.bob,1_000_000), Ok(()));
            assert_eq!(raffle.is_participating(accounts.bob), true);
        }
    }
}
