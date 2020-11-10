#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffle {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::{
            //HashMap as inkMap,
            Vec as InkVec,
        }
    };

    //A user can send in anywhere between 0.01 and 0.1 tokens.
    const DEPOSIT_MIN: u128 =  10_000_000_000_000;
    const DEPOSIT_MAX: u128 = 100_000_000_000_000;

    // countdown only starts once there are at least RAFFLE_TRIGGER players in the pool
    const RAFFLE_TRIGGER: u32 = 5; 

    /// The collected money from the pot is 
    /// automatically sent to a pre-defined address when the second winner is drawn.
    //const SEND_POT_ADDRESS: AccountId = 0xfff;

    /// The Raffle error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not DEPOSIT_MIN < payment < DEPOSIT_MAX
        EndowmentOutOfLimits,

        /// Returned if account already in the game
        AlreadyParticipating,
    }

    /// The Raffle result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// This is the storage of Raffle contract.
    #[ink(storage)]
    #[derive()]
    pub struct Raffle {
        owner: AccountId,
        total_balance: Balance,
        draw_allowed: bool,
        participant_list: InkVec<AccountId>,
        winner_list: InkVec<AccountId>,
    }

    /// Event emitted when new participant enters the raffle.
    #[ink(event)]
    pub struct NewParticipant {
        #[ink(topic)]
        participant: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when a winner is drawn.
    #[ink(event)]
    pub struct RaffleWinner {
        #[ink(topic)]
        participant: Option<AccountId>,
        #[ink(topic)]
        index: u32,
    }
    

    impl Raffle {
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            let instance = Self { 
                owner,
                total_balance: 0 as Balance,
                draw_allowed: false,
                participant_list: InkVec::new(),
                winner_list: InkVec::new(),
             };
             instance
        }

        /// A message that can be called on instantiated contracts.
        /// This one accepts new participant
        /// If amount is not within limits, it is rejected
        #[ink(message)]
            //let participant = self.env().caller();
        pub fn participate(&mut self, participant: AccountId, value: u128) -> Result<()>{
            
            //let participant = self.env().caller();
            ink_env::debug_println( "New participant" );

            if value < DEPOSIT_MIN || value > DEPOSIT_MAX {
                return Err(Error::EndowmentOutOfLimits)
            }
            
            if self.is_participating(participant) {
                return Err(Error::AlreadyParticipating)
            }
            self.participant_list.push(participant);
            self.total_balance += value;
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

        /// Draw winner
        #[ink(message)]
        pub fn draw_winner(&mut self) -> Result<()> {
            let winner_index: u32 = self.get_random_index();
            let winner = *self.participant_list.get(winner_index).unwrap();
            self.winner_list.push(winner);
            self.env().emit_event(RaffleWinner { participant: Some(winner), index: winner_index });
            Ok(())
        }
        
        fn get_random_index(&self) -> u32 {
            let random_index: u32 = Self::get_random_number();
            random_index % self.participant_list.len()
        }
        
        
        /// Check number of participants
        #[ink(message)]
        pub fn get_num_participants(&self) -> u32 {
            self.participant_list.len() 
        }
        
        /// Check raffle balance
        #[ink(message)]
        pub fn total_balance(&self) -> u128 {
            self.total_balance
        }
        
        // Thanks to @LaurentTrk#4763 on discord for get_random_number()
        // I wouldn't make on time without this
        // It is up to polkadot-hello-world-jury to decide if my submission is legit
        fn get_random_number() -> u32 {
            let seed: [u8; 8] = [7, 8, 9, 10, 11, 12, 13, 14];
            let random_hash = Self::env().random(&seed);
            Self::as_u32_be(&random_hash.as_ref())
        }
        fn as_u32_be(arr: &[u8]) -> u32 {
            ((arr[0] as u32) << 24)
                + ((arr[1] as u32) << 16)
                + ((arr[2] as u32) << 8)
                + ((arr[3] as u32) << 0)
        }
    }


    #[cfg(test)]
    mod tests {
        use ink_lang as ink;
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn default_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let raffle = Raffle::new(accounts.alice);
            assert_eq!(raffle.get_num_participants(), 0);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn test_participate() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);
            assert_eq!(raffle.participate(accounts.bob, DEPOSIT_MIN + 1), Ok(()));
            assert_eq!(raffle.is_participating(accounts.bob), true);

            // Expect one emitted event:
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
        }

        /// A user can send in anywhere between 0.01 and 0.1 tokens.
        #[test]
        fn test_endowment() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);

            assert_eq!(raffle.participate(accounts.charlie, DEPOSIT_MIN - 1), Err(Error::EndowmentOutOfLimits));
            assert_eq!(raffle.is_participating(accounts.charlie), false);

            assert_eq!(raffle.participate(accounts.charlie, DEPOSIT_MAX + 1), Err(Error::EndowmentOutOfLimits));
            assert_eq!(raffle.is_participating(accounts.charlie), false);
        }

        /// 15 minute countdown only starts once there are at least 5 players in the pool.
        #[ink::test]
        fn test_draw() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);
            assert_eq!(raffle.participate(accounts.alice, DEPOSIT_MIN + 1), Ok(()));
            assert_eq!(raffle.participate(accounts.bob, DEPOSIT_MIN + 1), Ok(()));
            assert_eq!(raffle.participate(accounts.charlie, DEPOSIT_MIN + 1), Ok(()));
            assert_eq!(raffle.participate(accounts.eve, DEPOSIT_MAX - 1), Ok(()));
            assert_eq!(raffle.draw_allowed, false);
            assert_eq!(raffle.participate(accounts.frank, DEPOSIT_MIN + 1), Ok(()));
            assert_eq!(raffle.draw_allowed, true);

            assert_eq!(raffle.draw_winner(), Ok(()));

            // Expect 5 newParticipant events and one draw
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 6);
        }

        /// A user can only play once.
        #[test]
        fn test_play_once() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);
            assert_eq!(raffle.participate(accounts.bob, DEPOSIT_MIN + 1), Ok(()));
            assert_eq!(raffle.participate(accounts.bob, DEPOSIT_MIN + 1), Err(Error::AlreadyParticipating));

        }
    }
}
