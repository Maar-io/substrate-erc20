#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod raffle {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{
        collections::{
            Vec as InkVec,
        }
    };

    //A user can send in anywhere between 0.01 and 0.1 tokens.
    const DEPOSIT_MIN: u128 =  10_000_000_000_000;
    const DEPOSIT_MAX: u128 = 100_000_000_000_000;

    // countdown only starts once there are at least RAFFLE_TRIGGER players in the pool
    const RAFFLE_TRIGGER: u32 = 3; 

    /// Number of rafflr winners
    const RAFFLE_WINNERS: u8 = 2;

    /// The Raffle error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not DEPOSIT_MIN < payment < DEPOSIT_MAX
        EndowmentOutOfLimits,

        /// Returned if account already in the game
        AlreadyParticipating,

        /// Returned if we have 2 winners
        RaffleFinished,

        /// Pot Transfer failed
        TransferError,

        /// Raffle does not have enough participants
        TooFewParticpants,
    }

    /// The Raffle result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// This is the storage of Raffle contract.
    #[ink(storage)]
    #[derive()]
    pub struct Raffle {
        pot_receiver: AccountId,
        total_balance: Balance,
        draw_allowed: bool,
        raffle_finished: bool,
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
        pub fn new(pot_receiver: AccountId) -> Self {
            let instance = Self { 
                pot_receiver,
                total_balance: 0 as Balance,
                draw_allowed: false,
                raffle_finished: false,
                participant_list: InkVec::new(),
                winner_list: InkVec::new(),
             };
             instance
        }

        /// A message that can be called on instantiated contracts.
        /// This one accepts new participant
        /// If amount is not within limits, it is rejected
        #[ink(message, payable)]
        pub fn participate(&mut self, participant: AccountId) -> Result<()>{
            
            // self.env().caller() can be anyone willing to pay. 
            // contract stores entered participant address
            let value = self.env().transferred_balance();

            ink_env::debug_println( "New participant");

            if value < DEPOSIT_MIN || value > DEPOSIT_MAX {
                return Err(Error::EndowmentOutOfLimits)
            }
            
            if self.raffle_finished {
                return Err(Error::RaffleFinished)
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
            if self.raffle_finished{
                return Err(Error::RaffleFinished)
            }
            if !self.draw_allowed{
                return Err(Error::TooFewParticpants)
            }
            let winner_index: u32 = self.get_random_index();
            let winner = *self.participant_list.get(winner_index).unwrap();
            
            self.winner_list.push(winner);
            if self.winner_list.len() == RAFFLE_WINNERS as u32 {
                self.raffle_finished = true;
                let result = self.transfer_pot();
                if !result {
                    return Err(Error::TransferError);
                }
            }
            self.env().emit_event(RaffleWinner { participant: Some(winner), index: winner_index });
            Ok(())
        }  
        
        fn transfer_pot(&mut self) -> bool{
            let result = self.env().transfer(self.pot_receiver, self.total_balance);
            if result == Ok(()) {
                return true;
            }
            false
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

        /// Winner list
        #[ink(message)]
        pub fn announce_winners(&self) -> (Option<AccountId>, Option<AccountId>) {
            (
                self.winner_list.first().map(|v| v.clone()),
                self.winner_list.last().map(|v| v.clone()),
            )
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
            assert_eq!(raffle.pot_receiver, accounts.alice);
        }

        /// We test a simple use case of our contract.
        #[test]
        fn test_participate() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);
            do_transfer(accounts.bob, None);
            assert_eq!(raffle.participate(accounts.bob), Ok(()));
            assert_eq!(raffle.is_participating(accounts.bob), true);
            
            // Expect one emitted event:
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
        }
        /// A user can send in anywhere between 0.01 and 0.1 tokens.
        #[test]
        fn test_deposit_limits() {
            let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.alice);
            
            do_transfer(accounts.bob, Some(DEPOSIT_MIN- 1));
            assert_eq!(raffle.participate(accounts.charlie), Err(Error::EndowmentOutOfLimits));
            assert_eq!(raffle.is_participating(accounts.charlie), false);
            
            do_transfer(accounts.bob, Some(DEPOSIT_MAX+ 1));
            assert_eq!(raffle.participate(accounts.charlie), Err(Error::EndowmentOutOfLimits));
            assert_eq!(raffle.is_participating(accounts.charlie), false);
            
            do_transfer(accounts.bob, None);
            assert_eq!(raffle.participate(accounts.charlie), Ok(()));
            assert_eq!(raffle.is_participating(accounts.charlie), true);
        }

        /// 15 minute countdown only starts once there are at least 5 players in the pool.
        #[ink::test]
        fn test_draw() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.charlie);
            set_all_participants(&mut raffle);

            assert_eq!(raffle.draw_winner(), Ok(()));
            assert_eq!(raffle.winner_list.len(), 1);
            //do_transfer(accounts.alice, None);
            //assert_eq!(raffle.draw_winner(), Ok(()));

            // Expect events: 5 NewParticipant events, 1 RaffleWinner,
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 6);
        }

        /// There are at least 5 players in the pool.
        #[ink::test]
        fn test_draw_not_enough_participants() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.charlie);
            do_transfer(accounts.alice, None);
            assert_eq!(raffle.participate(accounts.alice), Ok(()));
    
            assert_eq!(raffle.draw_winner(), Err(Error::TooFewParticpants));

            // Expect events: 1 NewParticipant event
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
        }
        

        /// A user can only play once.
        #[test]
        fn test_play_once() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            
            let mut raffle = Raffle::new(accounts.charlie);
            do_transfer(accounts.alice, None);
            assert_eq!(raffle.participate(accounts.alice), Ok(()));
            do_transfer(accounts.alice, None);
            assert_eq!(raffle.participate(accounts.alice), Err(Error::AlreadyParticipating));

        }

        fn set_all_participants(raffle: &mut Raffle) {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            do_transfer(accounts.alice, None);
            assert_eq!(raffle.participate(accounts.alice), Ok(()));
            do_transfer(accounts.bob, None);
            assert_eq!(raffle.participate(accounts.bob), Ok(()));
            do_transfer(accounts.charlie, None);
            assert_eq!(raffle.participate(accounts.charlie), Ok(()));
            do_transfer(accounts.eve, None);
            assert_eq!(raffle.participate(accounts.eve), Ok(()));
            //assert_eq!(raffle.draw_allowed, false);
            do_transfer(accounts.frank, None);
            assert_eq!(raffle.participate(accounts.frank), Ok(()));
            //assert_eq!(raffle.draw_allowed, false);
        }
        
        fn do_transfer(caller: AccountId, amount: Option<Balance>){
            
            // Get contract address.
            let callee: [u8; 32] = [0x07; 32];

            let mut data =
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&caller);
            // Push the new execution context.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                caller,
                callee.into(),
                1_000_000,
                amount.unwrap_or(DEPOSIT_MIN),
                data,
            );
        }
            
    }
}
