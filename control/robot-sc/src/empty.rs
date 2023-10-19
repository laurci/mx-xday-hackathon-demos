#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait RobotContract {
    #[view(getDeposit)]
    #[storage_mapper("deposit")]
    fn deposit(&self, player: &ManagedAddress) -> SingleValueMapper<BigUint>;

    #[init]
    fn init(&self) {}

    #[endpoint]
    #[payable("EGLD")]
    fn join(&self) {
        let payment = self.call_value().egld_value();

        let caller = self.blockchain().get_caller();
        self.deposit(&caller)
            .update(|deposit| *deposit += &*payment);
    }
}
