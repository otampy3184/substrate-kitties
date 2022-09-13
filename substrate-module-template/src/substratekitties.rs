use support::{decl_storage, decl_module, StorageMap, dispatch::Result};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};
use parity_code::{Encode, Decode};


// Kitty用のランタイムカスタム構造体を作成
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64,
}

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // StrageとGetter関数を実装していく

        //Mappingストレージ
        Value: map T::AccountId => u64;
    }
}

decl_module! {
    trait struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Public関数を実装していく
        fn set_value(origin, value: u64) -> Result {
            //originをチェックしてメッセージが有効なアカウントで署名されているか確認
            let sender = ensure_signed(origin)?;

            //ランタイムストレージにu64の値を格納する
            <Value<T>>::put(value);

            Ok(())
        }

        // 新規でKittyを作成する関数
        fn create_kitty(origin) -> Result {
            // originの事前チェック
            let sender = ensure_signed(origin)?;

            // Kittyオブジェクトを使ってnew_kittyを作成する
            // new_kittyの中身にRuntimeストレージのデータを初期化↓データを入れる
            let new_kitty = Kitty {
                // 以下の記述を行うことでT::HashとT::Balanceが初期化される
                id: <T as system::Trait>::Hashing::hash_of(&0),
                dna: <T as system::Trait>::Hashing::hash_of(&0),
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            }

            Ok(())
        }
    }
}
