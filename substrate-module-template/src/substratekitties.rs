use support::{decl_storage, decl_module};

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // StrageとGetter関数を実装していく
    }
}

decl_module! {
    trait struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Public関数を実装していく
    }
}
