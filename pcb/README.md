# 基板の設計データ
KiCad 6.0.4 を用いて設計しています。

## 販売されているモデルとの違い
裏面のロゴデータが含まれていないことが唯一の違いです。

※表面の "RustyKeys" ロゴは含まれています。

## パーツリスト(BOM)
調達元は参考情報です。同型・互換の製品であれば他のパーツを使ってもよいでしょう。

- Raspberry Pi Pico x2　(調達元: [游舎工房](https://shop.yushakobo.jp/products/raspberry-pi-pico), [秋月電子通商](https://akizukidenshi.com/catalog/g/gM-16132/))
- 1N4148 x6 (調達元: [游舎工房](https://shop.yushakobo.jp/products/a0800di-01-100), [秋月電子通商](https://akizukidenshi.com/catalog/g/gI-16603/))
- L字ピンヘッダ(3P) x1 (調達元: [秋月電子通商](https://akizukidenshi.com/catalog/g/gC-15510/))
- ジャンパ線(ソケット-ソケット 3P) x1 (調達元: [秋月電子通商](https://akizukidenshi.com/catalog/g/gC-15384/))
- ピンヘッダ(40P) x1 (調達元: [秋月電子通商](https://akizukidenshi.com/catalog/g/gC-00167/))
  - 20P x1 でも代用できます (調達元: [秋月電子通商](https://akizukidenshi.com/catalog/g/gC-16590/))
- ゴム足 x4 (調達元: [Amazon.co.jp](https://www.amazon.co.jp/dp/B07QKFW6F6))
  - 直径 5mm x 高さ 2mm 程度のものなら代用できます
  - TRUSCO の TP6R-90 もおすすめです (調達元: [Amazon.co.jp](https://www.amazon.co.jp/dp/B01B4CQVFK))

## 利用しているシンボル・フットプリントライブラリ

- [Keyswitch Kicad Library](https://github.com/perigoso/keyswitch-kicad-library)
- [KiCad-RP-Pico](https://github.com/ncarandini/KiCad-RP-Pico)
