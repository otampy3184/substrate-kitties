use support::{decl_storage, decl_module, StorageValue, StorageMap,
    dispatch::Result, ensure, decl_event};
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

pub trait Trait: balances::Trait {
    // 外部用のEventタイプを定義
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// 外部発信用のイベント
decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
    {
        // 作成時用のイベント
        Created(AccountId, Hash),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // idをKittiyオブジェクトにマッピングする新しいKittiesストレージ
        Kitties get(kitty): map T::Hash => Kitty<T::Hash, T::Balance>;
        // kittyを所有するアカウントIDにkittyidをマッピングするKittyOwnerストレージ
        KittyOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        // 全Kiityをリストとして追跡するためのAllKittiesArray
        AllKittiesArray get(kitty_by_index): map u64 => T::Hash;
        // AllKittiesArrayで利用するCount
        AllKittiesCount get(all_kitties_count): u64;
        // AllkittiesをIndexとして管理するAllKittiesIndex
        AllKittiesIndex: map T::Hash => u64;

        // KittyIdから所有アカウントを特定するOwnedKittiesArrayストレージ
        // タプルを使って高次配列にし、一ユーザーが複数のKittyを持てるようにする
        OwnedKittiesArray get(kitty_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedKittiesCount get(owned_kitty_count): map T::AccountId => u64;
        OwnedKittiesIndex: map T::Hash => u64;

        // 一意の数字Nonce
        Nonce: u64;
    }
}

decl_module! {
    // Public関数を実装していく
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Eventをデポジットするための関数
        // ランタイム開発の一般的なパターン
        fn deposit_event<T>() = default;

        fn create_kitty(origin) -> Result {
            // originをチェックしてメッセージが有効なアカウントで署名されているか確認
            let sender = ensure_signed(origin)?;

            // KittyCountを取得し、オーバーフローチェックをしてから1インクリメントする
            let owned_kitty_count = Self::owned_kitty_count(&sender);

            let new_owned_kitty_count = owned_kitty_count.checked_add(1)
            .ok_or("Overflow adding a new kitty count to owned kitty count")?;

            // all_kitties_countを使って現在のKittyの数をえる
            let all_kitties_count = Self::all_kitties_count();

            // Kittyを新しく追加するため、AllKittiysCountを一つ増やすした値を作成する
            // インクリメント時は必ずchecked_add()を使ってオーバーフローを検知すること
            let new_all_kitties_count = all_kitties_count.checked_add(1)
            .ok_or("overflow adding a new kitty to total supply")?;

            // random_seedを使ってランダムハッシュを作成
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            //KittyOwnerストレージを使ってKittyの所有権を確認する
            ensure!(!<KittyOwner<T>>::exists(random_hash), "Kitty already exists");

             // Kittyオブジェクトを使ってnew_kittyを作成する
            // new_kittyの中身にRuntimeストレージのデータを初期化↓データを入れる
            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            };

            // 作成したKittyをストレージに加えていく
            <Kitties<T>>::insert(random_hash, new_kitty);
            <KittyOwner<T>>::insert(random_hash, &sender);

            // GobalKittyTrackingのストレージを更新していく
            // まずAllKittiesArrayのマッピングリストに登録したKittyを追加
            <AllKittiesArray<T>>::insert(all_kitties_count, random_hash);
            // 新たなKittyCountの値を追加する
            <AllKittiesCount<T>>::put(new_all_kitties_count);
            // 逆向きのマッピングリストン追加
            <AllKittiesIndex<T>>::insert(random_hash, all_kitties_count);

            // 所有しているKittyの情報を更新する
            <OwnedKittiesArray<T>>::insert((sender.clone(), owned_kitty_count), random_hash);
            <OwnedKittiesCount<T>>::insert(&sender, new_owned_kitty_count);
            <OwnedKittiesIndex<T>>::insert(random_hash, owned_kitty_count);

            // Nonceを一つ増やす
            <Nonce<T>>::mutate(|n| *n += 1);

            // Eventを呼び出す(create時に使ったアドレスとランダムハッシュを渡す)
            Self::deposit_event(RawEvent::Created(sender, random_hash));

            Ok(())
        }
    }
}