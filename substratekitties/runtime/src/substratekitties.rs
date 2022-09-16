use support::{decl_storage, decl_module, StorageValue, StorageMap,
    dispatch::Result, ensure, decl_event, traits::Currency};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};
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
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance
    {
        // 各種イベント
        Created(AccountId, Hash),
        PriceSet(AccountId, Hash, Balance),
        Transferred(AccountId, AccountId, Hash),
        Bought(AccountId, AccountId, Hash, Balance),
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
            // originを確認
            let sender = ensure_signed(origin)?;
            
            // random_seedを使ってランダムハッシュを作成
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            // Kittyオブジェクトを使ってnew_kittyを作成する
            // new_kittyの中身にRuntimeストレージのデータを初期化↓データを入れる
            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            };

            // リファクタリングしたMintを使う
            Self::mint(sender, random_hash, new_kitty)?;

            // Nonceを一つ増やす
            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }

        // 価格を設定するモジュール
        fn set_price(origin, kitty_id: T::Hash, new_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // Kittyが存在しているか確認する
            ensure!(<Kitties<T>>::exists(kitty_id), "This cat does not exist");

            // Owenerの所有権を確認する
            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this cat");

            let mut kitty = Self::kitty(kitty_id);

            // 新しい価格の代入し、ストレージの情報をアップデート
            kitty.price = new_price;
            <Kitties<T>>::insert(kitty_id, kitty);

            Self::deposit_event(RawEvent::PriceSet(sender, kitty_id, new_price));
            
            Ok(())
        }

        // Kittyの所有権を移転するモジュール
        fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty");

            Self::transfer_from(sender, to, kitty_id)?;

            Ok(())
        }

        // 売りに出されたKittyを購入する関数
        fn buy_kitty(origin, kitty_id: T::Hash, max_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // 存在確認
            ensure!(<Kitties<T>>::exists(kitty_id), "This cat does not exits");

            // Owenerの所有権を確認する
            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner != sender, "You can't buy your own cat");

            let mut kitty = Self::kitty(kitty_id);

            // Zero Traitsを使ってkittyの値段が0でない(＝売りに出されている)か調べる＋言い値以下かチェック
            let kitty_price = kitty.price;
            ensure!(kitty_price.is_zero(), "This cat you want to buy is not for sale");
            ensure!(kitty_price <= max_price, "The cat you want to buy is costs more than your max price");

            // Balanceモジュールのtransfer()を使って資金を移転する
            <balances::Module<T> as Currency<_>>::transfer(&sender, &owner, kitty_price)?;
            
            // ACTION: Transfer the kitty using `transfer_from()` including a proof of why it cannot fail
            Self::transfer_from(owner.clone(), sender.clone(), kitty_id)
            .expect("`owner` is shown to own the kitty; \
            `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
            `all_kitty_count` shares the same type as `owned_kitty_count` \
            and minting ensure there won't ever be more than `max()` kitties, \
            which means transfer cannot cause an overflow; \
            qed");


            // kittyを市場から戻す
            kitty.price = <T::Balance as As<u64>>::sa(0);
            <Kitties<T>>::insert(kitty_id, kitty);

            // Event発行
            Self::deposit_event(RawEvent::Bought(sender, owner, kitty_id, kitty_price));

            Ok(())
        }
    }
}

// mintとTransferを
impl<T: Trait> Module<T> {
    fn mint(to: T::AccountId, kitty_id: T::Hash, new_kitty: Kitty<T::Hash, T::Balance>) -> Result {
        // originをチェックしてメッセージが有効なアカウントで署名されているか確認
        ensure!(!<KittyOwner<T>>::exists(kitty_id), "Kitty already exists");
        // KittyCountを取得し、オーバーフローチェックをしてから1インクリメントする
        let owned_kitty_count = Self::owned_kitty_count(&to);

        let new_owned_kitty_count = owned_kitty_count.checked_add(1)
        .ok_or("Overflow adding a new kitty count to owned kitty count")?;

        // all_kitties_countを使って現在のKittyの数をえる
        let all_kitties_count = Self::all_kitties_count();

        // Kittyを新しく追加するため、AllKittiysCountを一つ増やすした値を作成する
        // インクリメント時は必ずchecked_add()を使ってオーバーフローを検知すること
        let new_all_kitties_count = all_kitties_count.checked_add(1)
            .ok_or("overflow adding a new kitty to total supply")?;



        //KittyOwnerストレージを使ってKittyの所有権を確認する
        ensure!(!<KittyOwner<T>>::exists(kitty_id), "Kitty already exists");

        // 作成したKittyをストレージに加えていく
        <Kitties<T>>::insert(kitty_id, new_kitty);
        <KittyOwner<T>>::insert(kitty_id, &to);

        // GobalKittyTrackingのストレージを更新していく
        // まずAllKittiesArrayのマッピングリストに登録したKittyを追加
        <AllKittiesArray<T>>::insert(all_kitties_count, kitty_id);
        // 新たなKittyCountの値を追加する
        <AllKittiesCount<T>>::put(new_all_kitties_count);
        // 逆向きのマッピングリストン追加
        <AllKittiesIndex<T>>::insert(kitty_id, all_kitties_count);

        // 所有しているKittyの情報を更新する
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count);


        // Eventを呼び出す(create時に使ったアドレスとランダムハッシュを渡す)
        Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }

    fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> Result {
        // Kittyがownerを持っているか確認してから代入
        let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;

        // 送信者の所有権を確認
        ensure!(owner == from, "from account does not owen this kitty");

        // 移転用の準備
        let owned_kitty_count_from = Self::owned_kitty_count(&from);
        let owned_kitty_count_to = Self::owned_kitty_count(&to);

        // 事前チェック
        let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1)
            .ok_or("overflow adding a new kitty to total supply")?;
        let new_owned_kitty_count_from = owned_kitty_count_from.checked_add(1)
            .ok_or("overflow adding a new kitty to total supply")?;

        // swap and popでトランスファーを行う
        let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);
        if kitty_index != new_owned_kitty_count_from {
            let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
            <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
            <OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
        }

        // 結果を記録していく
        <KittyOwner<T>>::insert(&kitty_id, &to);
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

        <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

        <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);
        
        Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));

        Ok(())
    }
}