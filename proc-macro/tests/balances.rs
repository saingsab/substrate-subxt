use codec::{
    Decode,
    Encode,
};
use frame_support::Parameter;
use sp_keyring::AccountKeyring;
use sp_runtime::traits::{
    AtLeast32Bit,
    MaybeSerialize,
    Member,
};
use std::fmt::Debug;
use substrate_subxt::{
    system::System,
    ClientBuilder,
    KusamaRuntime,
};
use substrate_subxt_proc_macro::{
    module,
    subxt_test,
    Call,
    Event,
    Store,
};

pub trait SystemEventsDecoder {
    fn with_system(&mut self) -> Result<(), substrate_subxt::EventsError>;
}

impl<T: System> SystemEventsDecoder for substrate_subxt::EventsDecoder<T> {
    fn with_system(&mut self) -> Result<(), substrate_subxt::EventsError> {
        Ok(())
    }
}

#[module]
pub trait Balances: System {
    type Balance: Parameter
        + Member
        + AtLeast32Bit
        + codec::Codec
        + Default
        + Copy
        + MaybeSerialize
        + Debug
        + From<<Self as System>::BlockNumber>;
}

#[derive(Clone, Decode, Default)]
pub struct AccountData<Balance> {
    pub free: Balance,
    pub reserved: Balance,
    pub misc_frozen: Balance,
    pub fee_frozen: Balance,
}

#[derive(Encode, Store)]
pub struct AccountStore<'a, T: Balances> {
    #[store(returns = AccountData<T::Balance>)]
    pub account_id: &'a <T as System>::AccountId,
}

#[derive(Call, Encode)]
pub struct TransferCall<'a, T: Balances> {
    pub to: &'a <T as System>::Address,
    #[codec(compact)]
    pub amount: T::Balance,
}

#[derive(Debug, Decode, Eq, Event, PartialEq)]
pub struct TransferEvent<T: Balances> {
    pub from: <T as System>::AccountId,
    pub to: <T as System>::AccountId,
    pub amount: T::Balance,
}

impl Balances for KusamaRuntime {
    type Balance = u128;
}

subxt_test!({
    name: transfer_test_case,
    runtime: KusamaRuntime,
    account: Alice,
    step: {
        state: {
            alice: AccountStore { account_id: &alice },
            bob: AccountStore { account_id: &bob },
        },
        call: TransferCall {
            to: &bob,
            amount: 10_000,
        },
        event: TransferEvent {
            from: alice.clone(),
            to: bob.clone(),
            amount: 10_000,
        },
        assert: {
            assert_eq!(pre.alice.free, post.alice.free - 10_000);
            assert_eq!(pre.bob.free, post.bob.free + 10_000);
        },
    },
});

#[async_std::test]
#[ignore]
async fn transfer_balance_example() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let client = ClientBuilder::<KusamaRuntime>::new().build().await?;
    let alice = AccountKeyring::Alice.to_account_id();
    let bob = AccountKeyring::Bob.to_account_id();

    let alice_account = client.account(&alice).await?.unwrap_or_default();
    let bob_account = client.account(&bob).await?.unwrap_or_default();
    let pre = (alice_account, bob_account);

    let result = client
        .xt(AccountKeyring::Alice.pair(), None)
        .await?
        .watch()
        .transfer(&bob.clone().into(), 10_000)
        .await?;

    assert_eq!(
        result.transfer()?,
        Some(TransferEvent {
            from: alice.clone(),
            to: bob.clone(),
            amount: 10_000,
        })
    );

    let alice_account = client.account(&alice).await?.unwrap_or_default();
    let bob_account = client.account(&bob).await?.unwrap_or_default();
    let post = (alice_account, bob_account);

    assert_eq!(pre.0.free, post.0.free - 10_000);
    assert_eq!(pre.1.free, post.1.free + 10_000);
    Ok(())
}
