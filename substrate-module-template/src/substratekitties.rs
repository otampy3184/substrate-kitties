use support::{decl_storage, decl_module, StorageValue};
use system::ensure_signed;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // StrageとGetter関数を実装していく
        Value: u64;
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
    }
}
