# FC wireframe renderer

`wire3d.c` draws independently authored 3D models as cyan lines on a dark
background. It uses integer rotation, reciprocal-table perspective, a CPU
pixel buffer and two CHR-RAM pattern tables. No commercial game code, model
or graphics data is included. The source is licensed under MIT by DAISUKE OBA.

Select one viewport before including `wire3d.c`: 64×48, 96×64 or 128×96.
Define **both** `WIRE3D_FC_WIDTH` and `WIRE3D_FC_HEIGHT`; the default is 64×48.
The viewport is centered on the 256×240 background. A larger viewport uses
more tiles and draws longer lines, so choose it using measurements of your scene.

```c
#define WIRE3D_FC_WIDTH 96
#define WIRE3D_FC_HEIGHT 64
#include "wire3d.c"

void main(void) {
    Wire3DFC_Init();
    Wire3DFC_BeginFrame();
    Wire3DFC_DrawLine2D(4,4,91,59);
    Wire3DFC_DrawLine2D(91,4,4,59);
    Wire3DFC_EndFrame();
    while(1) {}
}
```

This example draws two crossing diagonals to check the chosen viewport and
line coverage. Compile this source once; it already includes the implementation.

```powershell
.\kitaqfc\kitaqfc.exe .\cross.c -I .\kitaqfc\lib --mapper=nrom --nes-chr-ram -o .\cross.nes
```

The ROM requires **8 KiB writable CHR RAM** and cartridge PRG RAM covering
`$6800–$70BF`. The renderer owns both pattern tables, nametable zero, scroll
and the background palette, and reserves zero-page `$00–$0A`. Do not overlap
those regions or change their mapper mapping while drawing. `--nes-chr-ram`
selects RAM instead of CHR ROM; it cannot be combined with an artwork file
through `--nes-chr` or `--chr-rom`, and is rejected for the CNROM profile.

The renderer is not reentrant. Initialization disables NMI and enables the
background after setup. Drawing and upload run in the foreground; do not run
another PPU writer or interrupt handler that changes its scratch memory.
An upload can span multiple display frames. This implementation does not
provide an asynchronous game-loop or audio-update scheduler.

| Function | Purpose and contract |
|---|---|
| `Wire3DFC_Init()` | Clear both CHR tables and CPU drawing buffers, build the centered tile map, select cyan-on-dark colors and enable the background with NMI disabled. Call before any frame operation. |
| `Wire3DFC_BeginFrame()` | Clear CPU tiles touched by the preceding drawing. The displayed image remains intact until `EndFrame` swaps buffers. Call once before constructing each complete image. |
| `Wire3DFC_DrawLine2D(ax,ay,bx,by)` | Draw an inclusive segment into the CPU buffer. Endpoints are signed, in −512…511. Pixels outside the viewport are clipped; entirely outside bounding boxes are rejected. In-bounds lines use a register-based 6502 Bresenham loop. |
| `Wire3DFC_RotatePoint(x,y,z,rx,ry,rz)` | Modify three distinct writable `s16` components in Y, X, Z order. Angles wrap in 32 steps per turn. Start with each component in −63…63. Q6 reductions truncate toward zero. Any null component pointer makes the call a no-op. |
| `Wire3DFC_ProjectPoint(x,y,z,sx,sy)` | Convert a camera-space point to screen coordinates. Require X/Y in −127…127 and depth in 32…255. Return 1 and write signed, unclipped coordinates on success; return 0 without changing outputs for invalid coordinates or null output pointers. Positive X is right and positive Y is up. |
| `Wire3DFC_DrawLine3D(ax,ay,az,bx,by,bz)` | Project both endpoints and draw their screen segment. If either endpoint fails the projection range, omit the entire edge. It does not intersect edges with the near or far plane. |
| `Wire3DFC_DrawModel(vertices,count,edges,edge_count,x,y,z,rx,ry,rz)` | Transform and project each of the first 24 vertices once, cache the results, then draw indexed edges. Vertices contain three `s8` components; edges contain two `u8` indices. Null arrays, invalid indices and rejected endpoints are skipped. Keep translated sums in `s16` range. All valid edges are drawn; this API does not perform hidden-face removal. |
| `Wire3DFC_EndFrame()` | Upload tiles touched by the staged image or by the previous contents of the hidden CHR buffer, including tiles that must be erased. Transfer at most 16 tiles per VBlank batch, restore scroll, and display the completed image at a fresh VBlank. Blocks until finished. |

After `EndFrame`, `wire3d_uploaded_tiles` reports uploaded tiles and
`wire3d_transfer_frames` reports upload batches plus the final swap wait.
The latter is **not** total elapsed rendering time: transformation, rasterization
and CPU bookkeeping also take time. Perspective uses a quantized reciprocal
table; its nearest 128-pixel-wide scale saturates from 256 to 255.

The cube sample in the HTML manual demonstrates vertex sharing, rotation,
projection, twelve indexed edges and complete-frame presentation. Its validation
images show a cube rotated about the Y axis; all twelve edges should remain
connected and no lines from a preceding image should remain.

## 日本語

`wire3d.c` は、独自に作成した頂点・辺のモデルを暗い背景にシアン色の線で描画します。
整数の回転計算、逆数表による透視投影、CPU側の描画バッファ、2面のCHR RAMを使います。
市販ゲームのコード・モデル・画像は含めていません。DAISUKE OBAによるMITライセンスのソースです。

画面サイズは64×48、96×64、128×96から選び、`wire3d.c`を読み込む前に
`WIRE3D_FC_WIDTH`と`WIRE3D_FC_HEIGHT`の両方を定義します。省略時は64×48です。
表示領域は256×240の背景の中央に置かれます。大きい領域ほど線が長く、転送するタイルも
増えるため、実際の場面を計測して選んでください。

上のサンプルは、選んだ表示領域に交差する2本の対角線を描きます。
ソース内で実装を読み込んでいるため、ビルド時に`wire3d.c`を重ねて指定しません。
ビルドには`--nes-chr-ram`を指定します。8 KiBのCHR RAMと、カートリッジ側の
`$6800–$70BF`を読み書きできるPRG RAMが必要です。ゼロページ`$00–$0A`も専有します。
CHRの両パターンテーブル、ネームテーブル0、スクロール、背景パレットを他の描画処理と共有しません。
`--nes-chr`や`--chr-rom`による画像ファイルの指定とは併用できず、CNROMでも使用できません。

| 関数 | 用途と条件 |
|---|---|
| `Wire3DFC_Init()` | CHRの両画面とCPU側バッファを消去し、中央のタイル配置と配色を設定します。NMIを無効にして背景を表示します。最初に1回呼びます。 |
| `Wire3DFC_BeginFrame()` | 直前の描画で使ったCPU側タイルを消します。表示中の画像は保持します。次の画像を組み立てる前に呼びます。 |
| `Wire3DFC_DrawLine2D(ax,ay,bx,by)` | 両端を含む線をCPU側バッファに描きます。端点は−512～511です。領域外の画素は描かず、両端が同じ側の外にある線は省略します。領域内の線は専用の6502ループで描きます。 |
| `Wire3DFC_RotatePoint(x,y,z,rx,ry,rz)` | 別々の`s16`変数を指す3本のポインターを渡し、Y・X・Zの順に回転します。32段階で1周し、開始時の各成分は−63～63に収めます。小数部分は0方向に切り捨てます。いずれかのポインターがNULLなら変更しません。 |
| `Wire3DFC_ProjectPoint(x,y,z,sx,sy)` | カメラ座標を画面座標へ変換します。X/Yは−127～127、奥行きは32～255です。成功時は1を返し、表示領域に切り詰める前の符号付き座標を書きます。範囲外や出力ポインターがNULLなら0を返し、出力は保持します。Xの正方向が右、Yの正方向が上です。 |
| `Wire3DFC_DrawLine3D(ax,ay,az,bx,by,bz)` | 両端を投影して線を描きます。一端でも投影条件に合わなければ線全体を省略します。近面・遠面との交点は計算しません。 |
| `Wire3DFC_DrawModel(vertices,count,edges,edge_count,x,y,z,rx,ry,rz)` | 先頭24頂点までを各1回だけ変換・投影して保存し、頂点番号を指定した辺を描きます。頂点は`s8`の3成分、辺は`u8`の頂点番号2個です。NULLの配列、不正な頂点番号、投影できない端点は無視します。平行移動の加算を`s16`の範囲に収めてください。隠面処理は行いません。 |
| `Wire3DFC_EndFrame()` | 非表示側に必要なタイルを書き込み、古い線があったタイルも消去します。1回のVBlankで最大16タイルを送り、転送完了後のVBlankで表示を切り替えます。完了まで待機します。 |

`wire3d_uploaded_tiles`は転送したタイル数、`wire3d_transfer_frames`は転送バッチ数と
最後の表示切り替え待機の合計です。頂点計算・線画・準備の時間を含む総フレーム数ではありません。
初期化はNMIを無効にし、描画と転送は呼び出した処理を専有します。
ゲームや音声を並行更新する仕組みは含まれません。割り込みからの再入や、他のPPU更新処理との
併用は避けてください。逆数表の値は整数化され、128幅の最も近い奥行きでは係数256を255に制限します。

HTML説明書の立方体サンプルは、頂点の共有、回転、投影、12本の辺、完成した画像の表示を確認するためのものです。
検証画像ではY軸回転した立方体の辺がつながり、前の画像の線が残らないことを確認します。
