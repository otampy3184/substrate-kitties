# Substratekitties

[Substrate Collectable]<https://substrate-developer-hub.github.io/substrate-collectables-workshop/>に従い、Substrateを使った独自のチェーンを作って実際に内部のランタイムを編集してカスタムを行う

# Enviroment:
- Ubuntu x86_64
- Node v14.15.4
- npm v6.14.10
- yarn v1.22.19
- rustup v1.25.1 

# How to start:

初期設定
```
$ curl https://getsubstrate.io -sSf | bash -s -- --fast
```

カスタムノード立ち上げ(サンプル)
```
$ git clone --branch v1.0 https://github.com/substrate-developer-hub/substrate-package
$ ./substrate-package-rename.sh substratekitties <some-project-name>
```

ノードのビルド(マシンの性能によっては時間がかかる)
```
$ cd substratekitties
$ ./scripts/init.sh
$ ./scripts/build.sh
$ cargo build --release
```

ノードの立ち上げ
```
$ ./target/release/substratekitties --dev
```
以上でSubstrateのサンプル用のノードが立ち上がる

# How to use it:

ランタイムのソースを更新した際は以下の処理を行い、チェーンのランタイムを更新する
```
$ cd substratekitties
$ ./scripts/build.~~
$ cargo build --release
$ ./target/release/substratekitties --dev
```

ブロックエクスプローラーは公式に用意されているpolkadot-js/appを使う
```
$ cd polkadot-js-app
$ yarn && yarn start
```
localhost:3000にエクスプローラーが立ち上がる