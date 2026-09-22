# danmaku

Copyright (c) 2026 DAISUKE OBA. SPDX-License-Identifier: MIT

## English

danmaku is a 64-bullet fixed-point pool. Initialize it, spawn bullets, call step for movement and collisions, call draw to build OAM, then perform DMA in VBlank. Handle player input, damage, scoring and audio in game code. The sample uploads an original diamond tile to CHR RAM; the library itself can also use CHR ROM sprites.

Reserve writable cartridge PRG RAM at $6000–$627F (640 bytes) for 64 bullets. Enable that RAM in the cartridge mapper. Shared state makes this library non-reentrant.

dm_count is the live bullet count and dm_peak its high-water mark. The 16-bit dm_spawned and dm_rejected counters track successful and failed spawns and wrap after 65535. dm_x and dm_y are center coordinates; ignore their values when dm_active is zero.

### `danmaku_reset`

Deactivate every bullet and reset hit/graze events, peak and lifetime counts, and allocation/draw-order cursors.


No return value.

Player position and dm_invulnerable are preserved. OAM and the display are not cleared; draw the empty pool and perform DMA, or hide OAM separately, to remove visible bullets.

```c
danmaku_reset();
```

### `danmaku_clear`

Deactivate every bullet and zero dm_count, dm_hit and dm_graze. Use this to remove bullets during play.


No return value.

Preserve dm_peak, dm_spawned, dm_rejected, allocation and draw-order cursors, and player settings. The sample spawns one bullet and clears it: the live count becomes zero while the lifetime spawn count remains one. Update OAM separately.

```c
danmaku_clear();
```

### `danmaku_spawn`

Search the pool cyclically for one free slot and initialize its position, velocity and ungrazed state. Spawning alone does not move it or test collisions.

- `x`: Horizontal center in pixels; accepted values are 8 through 247.
- `y`: Vertical center in pixels; accepted values are 24 through 231.
- `vx`: Horizontal velocity in signed Q4.4. A value of 16 moves one pixel per update; −16 moves one pixel in the opposite direction. Values −128..127 represent −8..+7.9375 pixels.
- `vy`: Vertical velocity in signed Q4.4. A value of 16 moves one pixel per update; −16 moves one pixel in the opposite direction. Values −128..127 represent −8..+7.9375 pixels.

Return 1 on success, or 0 for an invalid position or a full 64-slot pool. Success increments dm_count and dm_spawned and updates dm_peak as needed; failure increments dm_rejected.

```c
danmaku_spawn(48,64,16,0);
```

### `danmaku_fan`

Spawn bullets while advancing direction by a fixed step. The step and count can produce a fan or a full ring.

- `x`: Horizontal center in pixels; accepted values are 8 through 247.
- `y`: Vertical center in pixels; accepted values are 24 through 231.
- `direction`: Initial direction in 32 steps per turn: 0 right, 8 down, 16 left, 24 up. Only the low five bits are used.
- `step`: Direction increment after each bullet, wrapping modulo 32. Zero sends every bullet in the same direction.
- `count`: Number of spawn attempts. Values above 64 are clamped to 64; zero does nothing.
- `speed`: Speed magnitude in sixteenths of a pixel per update: 16 is one pixel and 32 is two. Values above 64 are clamped to 64.

No return value.

Velocities come from an integer sine table and truncate toward zero, so diagonal speeds and radii are quantized. A full pool does not cancel remaining attempts; each failure increments dm_rejected.

```c
danmaku_fan(160,104,0,2,16,32);
```

### `danmaku_step`

Advance each live bullet once, remove offscreen bullets and evaluate player hits and grazing. Bullet time advances with calls, not elapsed wall-clock time.


No return value.

Set dm_player_x and dm_player_y to the player center before calling. After movement, distances of at most three pixels on both axes count as a hit. If dm_invulnerable is zero, set dm_hit to one and remove the bullet; invulnerability leaves it alive.

Outside the hit box, distances of at most eleven pixels on both axes produce one graze per bullet. dm_hit and dm_graze are reset at entry and describe only this update. A bullet inside the hit box while the player is invulnerable does not count as a graze.

This function neither writes OAM nor calls damage or sound callbacks. Read dm_hit and dm_graze and update player state or scoring in game code.

```c
danmaku_step();
```

### `danmaku_draw`

Build 8×8 sprites from bullet centers and write the requested shadow-OAM range, without moving bullets or testing collisions.

- `first_oam`: First OAM slot (0..63). Values of 64 or more return zero without writing. The sample starts at one to reserve slot zero for the player.
- `slots`: Number of consecutive available slots, clamped to 64−first_oam. Zero writes nothing but still advances draw priority.
- `tile`: 8×8 pattern tile shared by all bullets. Configure 8×8 sprite mode; this call uses the same shape for every bullet.
- `attributes`: NES sprite attributes shared by all bullets: bits 0–1 palette, bit 5 behind background, bit 6 horizontal flip, bit 7 vertical flip.

Return the number of bullets written to OAM: the smaller of the live count and available slots.

Advance the starting bullet slot by 13 modulo 64 on each call so fixed bullets do not always get priority. Hide unused slots within the requested range with Y=240, preserving OAM outside that range.

Subtract four from center X and five from center Y; the hardware adds one to OAM Y, centering the sprite. Call __oam_dma during VBlank yourself. The hardware limit of eight sprites per scanline still applies.

```c
bullet_result[9]=danmaku_draw(1,24,1,0);
```

### Complete example

[HTML](https://bartaro.github.io/kitaq-docs/en/fc-library.html#module-danmaku)

The four yellow bullets on the left have moved 16 pixels in the cardinal directions from (48,64). The sixteen yellow bullets on the right form a ring roughly 32 pixels from (160,104). The cyan player at (112,192) owns reserved OAM slot zero. This example holds the image after sixteen logic updates.

```powershell
New-Item -ItemType Directory -Force .\out | Out-Null
.\kitaqfc\kitaqfc.exe .\kitaq-docs\samples\api-examples\fc\danmaku_patterns.c -I .\kitaqfc\lib --mapper=nrom --nes-chr-ram -o .\out\danmaku_patterns.nes --no-cache --no-disasm
.\kurosaki\kurosaki.exe run .\out\danmaku_patterns.nes --headless --frames 180 --png .\out\danmaku_patterns.png
```

KUROSAKI checks four compiler configurations against all pixels, bullet counts and positions, reserved OAM and rotating draw order. Separate numeric checks cover hits, grazing, offscreen removal and a full pool. Hardware has not been tested.

## 日本語

danmakuは64発の固定小数点弾プールです。初期化→発生→stepで移動・判定→drawでOAM作成→VBlankでDMAの順に使います。自機の入力、ダメージ、得点、音はゲーム側で処理します。サンプルは独自に作成したひし形のタイルをCHR RAMへ書きますが、ライブラリ自体はCHR ROMでも使えます。

64発の弾のため、書き込み可能なカートリッジPRG RAM $6000～$627F（640バイト）を予約します。使用するマッパーでRAMを有効にしてください。共有状態を使うため再入できません。

dm_countは生存弾数、dm_peakは最大生存弾数です。dm_spawnedとdm_rejectedは成功・失敗した発生数を16ビットで数え、65535の次は0へ戻ります。dm_x・dm_yは弾の中心座標で、dm_activeが0のスロットの値は使わないでください。

### `danmaku_reset`

全弾を無効にし、命中・かすり・最大弾数・発生数の統計と、割り当て・表示順の巡回位置を0へ戻します。


戻り値はありません。

dm_player_x、dm_player_y、dm_invulnerableは保持します。OAMや画面は消さないため、画面上の弾を取り除くには空になったプールをdrawしてDMAするか、別途OAMを隠します。

```c
danmaku_reset();
```

### `danmaku_clear`

全弾を無効にし、dm_count・dm_hit・dm_grazeを0にします。プレイ中に弾だけを消す用途です。


戻り値はありません。

dm_peak、dm_spawned、dm_rejected、割り当て位置と表示順の巡回位置、自機設定を保持します。サンプルでは1発発生させてclearした後、生存数は0、累計発生数は1になります。OAMの更新は別途必要です。

```c
danmaku_clear();
```

### `danmaku_spawn`

空きスロットを巡回して1発を割り当て、位置・速度と未かすり状態を設定します。発生した時点では移動や命中判定をしません。

- `x`: 発生位置の横中心座標です。8以上248未満のピクセル値を指定します。
- `y`: 発生位置の縦中心座標です。24以上232未満のピクセル値を指定します。
- `vx`: 横方向の速度です。符号付きQ4.4で、16は1更新あたり1ピクセル、−16は逆向き1ピクセルです。値域−128～127は−8～+7.9375ピクセルに相当します。
- `vy`: 縦方向の速度です。符号付きQ4.4で、16は1更新あたり1ピクセル、−16は逆向き1ピクセルです。値域−128～127は−8～+7.9375ピクセルに相当します。

成功時は1、位置が範囲外または64スロットすべて使用中なら0です。成功時はdm_count・dm_spawnedを増やし、必要ならdm_peakを更新します。失敗時はdm_rejectedを増やします。

```c
danmaku_spawn(48,64,16,0);
```

### `danmaku_fan`

方向を一定量ずつ進めながら複数の弾を発生させます。刻みと個数で扇形や全周の弾を作れます。

- `x`: 発生位置の横中心座標です。8以上248未満のピクセル値を指定します。
- `y`: 発生位置の縦中心座標です。24以上232未満のピクセル値を指定します。
- `direction`: 最初の方向です。32段階で1周し、0=右、8=下、16=左、24=上です。下位5ビットを使います。
- `step`: 1発ごとに加える方向の刻みです。加算後は32で折り返します。0ならすべて同方向です。
- `count`: 発生を試みる弾数です。64を超える値は64になり、0では何もしません。
- `speed`: 速度の大きさを1/16ピクセル単位で指定します。16は1、32は2ピクセル/更新です。64を超える値は64になります。

戻り値はありません。

整数の正弦表から速度を求め、0方向へ丸めます。斜め方向の速度や半径には量子化誤差があります。満杯でも残りの発生試行を続け、失敗分をdm_rejectedに加えます。

```c
danmaku_fan(160,104,0,2,16,32);
```

### `danmaku_step`

有効な弾を1回移動し、画面外の弾を除去して、自機との命中・かすりを判定します。呼び出し回数が弾の時間を進めます。


戻り値はありません。

呼び出す前にdm_player_x・dm_player_yへ自機の中心を指定します。移動後の位置で、各軸の距離が3以下なら命中です。dm_invulnerableが0のときdm_hitを1にして弾を消し、無敵なら弾を残します。

命中範囲外で各軸の距離が11以下なら、弾ごとに1回だけかすりを数えます。dm_hitとdm_grazeは呼び出しの冒頭で0に戻る、その更新回だけの結果です。無敵で命中範囲内にある弾はかすりとして数えません。

この処理はOAMを書かず、ダメージ処理や効果音のコールバックも呼びません。dm_hit・dm_grazeを読んで、ゲーム側で自機の状態や得点を更新します。

```c
danmaku_step();
```

### `danmaku_draw`

弾の中心座標から8×8スプライトを組み立て、指定したシャドーOAM範囲へ書きます。弾の移動や命中判定は行いません。

- `first_oam`: 先頭OAMスロット番号（0～63）です。64以上なら書かずに0を返します。サンプルは0を自機用に予約して1から使います。
- `slots`: 使用できる連続スロット数です。残りの64−first_oam個に制限します。0なら書き込みませんが表示順の巡回位置は進みます。
- `tile`: 全弾に使う8×8パターンのタイル番号です。スプライトを8×8モードにして、同じ形の弾に使います。
- `attributes`: 全弾に使うNESスプライト属性です。下位2ビットはパレット、ビット5は背景の後ろ、6は左右反転、7は上下反転です。

実際にOAMへ書いた弾数です。生存弾数と有効なスロット数の小さい方になります。

開始する弾スロットを呼び出しごとに13ずつ、64で折り返して進めます。固定した弾だけが常に優先されることを避けます。指定範囲の未使用スロットはY=240で隠し、範囲外のOAMは保持します。

中心からXは4、OAMのYは5を引き、ハードウェアの表示Y+1を含めて中央を合わせます。DMAは呼び出さないため、VBlankで__oam_dmaを実行してください。1走査線8スプライトというハードウェアの上限は残ります。

```c
bullet_result[9]=danmaku_draw(1,24,1,0);
```

### 完全なサンプル

[HTML](https://bartaro.github.io/kitaq-docs/fc-library.html#module-danmaku)

左側の黄色い4発は中心(48,64)から上下左右へ16ピクセル移動した単発弾です。右側の黄色い16発は中心(160,104)から約32ピクセル広がった円形弾です。水色の自機は(112,192)で、OAMスロット0を予約しています。画像は16回の更新後に止めてあります。

```powershell
New-Item -ItemType Directory -Force .\out | Out-Null
.\kitaqfc\kitaqfc.exe .\kitaq-docs\samples\api-examples\fc\danmaku_patterns.c -I .\kitaqfc\lib --mapper=nrom --nes-chr-ram -o .\out\danmaku_patterns.nes --no-cache --no-disasm
.\kurosaki\kurosaki.exe run .\out\danmaku_patterns.nes --headless --frames 180 --png .\out\danmaku_patterns.png
```

KUROSAKIで4種類のコンパイル設定を実行し、全画素、弾の数と位置、予約したOAM、表示順の回転を照合しています。命中・かすり・画面外除去・満杯の処理は別の数値検証でも確認しています。実機での検証ではありません。
