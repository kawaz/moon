# MoonBit ライブラリ出力機能の調査・実装ログ

## 概要

MoonBitパッケージ（`is-main: false`）からネイティブライブラリ（共有ライブラリ `.so` / 静的ライブラリ `.a`）を出力する機能を調査・実装した。

## ブランチ

- `claude/enable-library-output-gsHey`

## コミット履歴

1. `b36945d` - feat: enable library output for is-main:false packages when explicitly targeted
2. `8016206` - feat: support native shared (.so) and static (.a) library output
3. `ddbe015` - fix: eliminate libbacktrace dependency from static library output

## 実装内容

### 1. NativeOutputType enum の追加

**ファイル**: `crates/moonutil/src/package.rs`

`moon.pkg.json` の `link.native` セクションに `output-type` フィールドを追加。
値は `"shared"`（デフォルト）または `"static"`。

```json
{
  "link": {
    "native": {
      "exports": ["my_add", "my_multiply"],
      "output-type": "static"
    }
  }
}
```

### 2. 共有ライブラリ (.so) 出力

既存の `lower_build_exe_regular()` がネイティブターゲットで `.so` を生成する仕組みを利用。
`-fPIC` フラグをランタイムコンパイルに追加して共有ライブラリへのリンクを可能にした。

**ファイル**: `crates/moonbuild-rupes-recta/src/build_lower/lower_aux.rs`

### 3. 静的ライブラリ (.a) 出力

新しい `lower_build_static_lib()` メソッドを追加。3ステップのビルドパイプライン：

1. MoonBitが生成した `.c` ファイルを `.o` にコンパイル
2. `runtime.c` を `-DMOONBIT_ALLOW_STACKTRACE` なしで再コンパイル（libbacktrace依存を排除）
3. すべての `.o` ファイルを `ar rcs` でアーカイブして `.a` を生成

**ファイル**: `crates/moonbuild-rupes-recta/src/build_lower/lower_build.rs`

### 4. 静的ライブラリのアーティファクトパス

**ファイル**: `crates/moonbuild-rupes-recta/src/build_lower/artifact.rs`

| OS      | 拡張子 |
|---------|--------|
| Linux   | `.a`   |
| macOS   | `.a`   |
| Windows | `.lib` |

## 技術的な知見

### MoonBit関数名のマングリング

エクスポートされた関数名はマングルされる：
- `my_add` → `_M0FP33poc13library__test5mylib7my__add`

C側から呼ぶ場合はマングルされた名前を使う必要がある。

### Tree Shaking

正常に動作することを確認。エクスポートリストに含まれない関数（`unused_factorial`, `unused_repeat`）はライブラリ出力から除外される。

### libbacktrace

- GCCのスタックトレースライブラリ
- `runtime.c` で `MOONBIT_ALLOW_STACKTRACE` マクロにより条件付きで使用される
- 共有ライブラリでは通常通り含まれる
- 静的ライブラリではこのマクロなしで再コンパイルすることで依存を排除

### -fPIC フラグ

Position Independent Code。共有ライブラリに必要。
ランタイムコンパイル時にネイティブターゲット（Linux/macOS）で常に追加するよう変更。
Windows では不要（PEフォーマットは元々位置独立）。

## クロスコンパイル

現在は非対応。`std::env::consts::OS` からハードコードされたOS判定を使用しており、`--target-os` のようなフラグは存在しない。

## LLVMバックエンド

### 状態: 検証不可

- moonc stable (`v0.8.3+cd28f524e`) → `LLVM backend is disabled`
- moonc dev (`v0.8.3+cd28f524e-dev`) → 同上
- LLVMバックエンドはmoonc内部でコンパイル時に無効化されている
- 公開チャンネルのmoonc バイナリでは利用不可

### LLVMの特徴（コードから分かること）

- Native は `.c` を経由するが、LLVM は moonc が直接 `.o` を生成する
- `link-core` ステップで `-target llvm` を指定
- `-llvm-opt`, `-llvm-target`, `-S`, `-emit-llvm-ir` などのオプションあり

### ビルドシステム側の対応

`moon`（ビルドシステム）側ではLLVMターゲットに対応する設計になっている。
moonc がLLVM link-coreを有効にすれば、そのまま動作する想定。

## POCプロジェクト構成

```
/tmp/poc-library-output/
├── moon.mod.json          # module: poc/library_test
├── math/
│   ├── moon.pkg.json
│   └── math.mbt           # add(), multiply(), unused_factorial()
├── utils/
│   ├── moon.pkg.json
│   └── utils.mbt           # greet(), unused_repeat()
├── mylib/
│   ├── moon.pkg.json       # exports: my_add, my_multiply, my_greet
│   └── mylib.mbt           # pub fn my_add/my_multiply/my_greet (ラッパー)
├── test_caller.c           # C言語からMoonBit関数を呼ぶテストプログラム
├── test_native_so           # 共有ライブラリとリンクしたテストバイナリ
└── test_native_a            # 静的ライブラリとリンクしたテストバイナリ
```

## テスト結果

### 共有ライブラリ (.so)
```
$ ./test_native_so
my_add(3, 4) = 7
my_multiply(5, 6) = 30
```

### 静的ライブラリ (.a)
```
$ ./test_native_a
my_add(3, 4) = 7
my_multiply(5, 6) = 30
```

### Tree Shaking 確認
```
$ nm -C test_native_a | grep -i factorial
(出力なし = 正しく除外されている)
```

## 残課題

1. **LLVM検証**: moonc開発チームがLLVM対応バイナリを公開次第、検証可能
2. **Windows DLL**: コード上は対応しているが、Windows環境での実テストは未実施
3. **macOS dylib**: コード上は対応しているが、macOS環境での実テストは未実施
4. **クロスコンパイル**: 現在非対応、将来的な対応が望ましい
5. **関数名マングリング**: Cから呼ぶ際にマングル名が必要で不便。`extern "C"` 的な仕組みがあると良い
