# RustWorks

For an english version of this README go to [README.md](README.md).

Une OS (éventuellement) pour la calculatrice NumWorks (modèle n0110).

## Setup pour développement

D'abord installez Rust en suivant [ces instructions](https://www.rust-lang.org/tools/install),
puis, ouvrez une terminal et exécutez les commandes suivantes:

```zsh
rustup target add thumbv7em-none-eabihf

# Ubuntu
sudo apt-get install gcc-arm-none-eabi binutils-arm-none-eabi 
sudo apt-get install dfu-util
# macOS
brew tap osx-cross/arm
brew install arm-gcc-bin
brew install dfu-util

rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install cargo-make

# Si vous avez un débuggeur STLink que vous pouvez connecter à la NumWorks, installez `probe-run`
# ou `cargo-embed` pour facilement débugger et `cargo-flash` si vous voulez seulement des vitesses
# de flash plus rapides.
cargo install probe-run
cargo install cargo-embed
cargo install cargo-flash
```

## DFU Flash

Complétez le setup, branchez votre calculatrice et mettez le en mode DFU (appuyez sur 6 et reset en
même temps), puis exécutez la commande suivante:
```zsh
cargo make dfu
```

## STLink

Si vous avez un débuggeur STLink (personellement j'utilise la STLink V3SET) vous pouvez flash
beaucoup plus rapidement en utilisant une des méthodes suivantes:

### Flash
```zsh
cargo flash --chip=stm32f730V8Tx
# Ou vous pouvez utiliser la commande suivante qui fait la même chose mais peut être plus facile a
# s'en rappeler:
cargo make flash
```

### Debug

En utilisant `cargo-embed` (conseillé):
```zsh
cargo embed
```

En utilisant `probe-rs`:
```zsh
cargo run
```

