use support::{decl_storage, decl_module, StorageMap, dispatch::Result};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};
use parity_codec::{Encode, Decode};

// Kitty用のランタイムカスタム構造体を作成
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64,
}

pub trait Trait: balances::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        //Mappingストレージ
        OwnedKitty get(kitty_of_owner): map T::AccountId => Kitty<T::Hash, T::Balance>;
    }
}

decl_module! {
    // Public関数を実装していく
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn create_kitty(origin) -> Result {
            // originをチェックしてメッセージが有効なアカウントで署名されているか確認
            let sender = ensure_signed(origin)?;
             // Kittyオブジェクトを使ってnew_kittyを作成する
            // new_kittyの中身にRuntimeストレージのデータを初期化↓データを入れる
            let new_kitty = Kitty {
                id: <T as system::Trait>::Hashing::hash_of(&0),
                dna: <T as system::Trait>::Hashing::hash_of(&0),
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            };

            <OwnedKitty<T>>::insert(&sender, new_kitty);

            Ok(())
        }
    }
}