#ifndef KBFC_H
#define KBFC_H

#include "intrinsics.h"

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned char bool;

define u8 FALSE = 0;
define u8 TRUE = 1;

extern u8 PPU_CTRL;
extern u8 PPU_MASK;
extern u8 PPU_STATUS;
extern u8 PPU_SCROLL;
extern u8 PPU_ADDRESS;
extern u8 PPU_DATA;
extern u8 OAM_ADDRESS;
extern u8 OAM_DMA;
extern u8 APU_FRAME_CTR;
extern u8 APU_STATUS;
extern u8 APU_DMC;
extern u8 APU_0;
extern u8 APU_1;
extern u8 APU_2;
extern u8 APU_3;
extern u8 APU_4;
extern u8 APU_5;
extern u8 APU_6;
extern u8 APU_7;
extern u8 APU_8;
extern u8 APU_A;
extern u8 APU_B;
extern u8 APU_C;
extern u8 APU_E;
extern u8 APU_F;

define u16 VRAM_NAMETABLE0 = 0x2000;
define u16 VRAM_PALETTES = 0x3F00;

define u8 TILE_EMPTY = 0x00;
define u8 TILE_FRAME = 0x01;
define u8 TILE_GRID = 0x02;
define u8 TILE_BLOCK_I = 0x03;
define u8 TILE_BLOCK_O = 0x04;
define u8 TILE_BLOCK_T = 0x05;
define u8 TILE_BLOCK_J = 0x06;
define u8 TILE_BLOCK_L = 0x07;
define u8 TILE_BLOCK_S = 0x08;
define u8 TILE_BLOCK_Z = 0x09;
define u8 TILE_BLOCK_COLOR3 = 0x0A;
define u8 TILE_DIGIT = 0x10;
define u8 TILE_LETTER = 0x20;
define u8 TILE_COLON = 0x3A;
define u8 TILE_DASH = 0x3B;
define u8 TILE_STAR = 0x3C;
define u8 TILE_DOT = 0x3D;

define u8 FIELD_X = 4;
define u8 FIELD_Y = 4;
define u8 FIELD_W = 10;
define u8 FIELD_H = 20;
define u8 FIELD_ATTR_W = 5;
define u8 FIELD_ATTR_H = 10;
define u8 FIELD_ATTR_EMPTY = 0xFF;
define u8 BOARD_STRIDE = 16;
define u8 BOARD_H = 22;
define u8 LOCK_FRAMES = 30;
define u8 SOFTLOCK_FRAMES = 10;

define u8 PANEL_X = 17;
define u8 PANEL_Y_NEXT = 5;
define u8 PANEL_Y_HOLD = 10;
define u8 PANEL_Y_SCORE = 15;
define u8 PANEL_Y_LEVEL = 20;
define u8 PANEL_Y_LINES = 24;

define u8 BG_ATTR_UI = 3;

define u8 MODE_SPLASH = 0;
define u8 MODE_TITLE = 1;
define u8 MODE_GAME = 2;
define u8 MODE_PAUSE = 3;
define u8 MODE_GAMEOVER = 4;

define u8 BUTTON_RIGHT = 0x01;
define u8 BUTTON_LEFT = 0x02;
define u8 BUTTON_DOWN = 0x04;
define u8 BUTTON_UP = 0x08;
define u8 BUTTON_START = 0x10;
define u8 BUTTON_SELECT = 0x20;
define u8 BUTTON_B = 0x40;
define u8 BUTTON_A = 0x80;

define u8 MINO_I = 0;
define u8 MINO_O = 1;
define u8 MINO_T = 2;
define u8 MINO_J = 3;
define u8 MINO_L = 4;
define u8 MINO_S = 5;
define u8 MINO_Z = 6;
define u8 MINO_NONE = 0xFF;

define u8 RANDOMIZER_BAG = 0;
define u8 RANDOMIZER_RETRO = 1;

struct Sprite
{
    u8 Y;
    u8 Char;
    u8 Attr;
    u8 X;
};

struct Sprites_Raw
{
    struct Sprite Sprites[64];
};

union Sprites
{
    struct Sprites_Raw Raw;
};

extern u8 CurrentMode;
extern u8 PreviousMode;
extern __hram u8 Pad0;
extern __hram u8 Pad0Prev;
extern __hram u8 InMainThread;
extern __hram u8 NmiFrameCounter;
extern __hram u8 AudioFrameCounter;

extern u8 PpuCtrl;
extern u8 PpuMask;
extern u8 PpuScrollX;
extern u8 PpuScrollY;
extern u8 PpuPalettes[0x20];
extern u8 PpuPalettesNew;
extern u8 NameAttributes[64];
extern u8 FieldAttrOwners[FIELD_ATTR_W * FIELD_ATTR_H];

extern u8 SettingStartLevel;
extern u8 SettingRandomizer;
extern u8 SettingNext;
extern u8 SettingHold;
extern u8 SettingMusic;
extern u8 SettingSound;

extern u8 Board[BOARD_H * BOARD_STRIDE];
extern u8 CurrentMino;
extern u8 CurrentRot;
extern u8 CurrentX;
extern u8 CurrentY;
extern u8 NextMino;
extern u8 HoldMino;
extern u8 HoldUsed;
extern u8 GameOverFlag;
extern u8 Level;
extern u8 Lines;
extern u16 Score;
extern u8 ScoreBCD[4];
extern u8 FrameCounter;
extern u8 RotateCooldown;
extern u8 DropBonus;
extern u8 LineClearActive;
extern u8 LineClearPhase;
extern u8 LineClearRows[4];
extern u8 DecimalTens;
extern u8 DecimalOnes;

extern __prg_rom u8 PaletteBg[];
extern __prg_rom u8 PaletteSp[];
extern __prg_rom u8 MinoShapes[];
extern __prg_rom u8 MinoTile[];
extern __prg_rom u8 MinoSpriteAttr[];
extern __prg_rom u8 GravityFrames[];

void main();
void BeginFrame();
void FlushBlitFrame();
void ResetSprites();
void HideAllSprites();
void UpdateInput();
void ReadInput();
u8 ConvertKitaqfcPadBits(u8 raw);
bool Button(u8 button);
bool ButtonDown(u8 button);
void SeedRandom(u8 seed);
u8 GetRandomByte();
void SplitDecimal10(u8 value);
void DisablePPU();
void EnablePPU();
void SetScroll(u8 x, u8 y);
void ClearBlitBuffer();
void CommitBlitBuffer();
void FlushBlitChunk();
void ProcessBlitBuffer();
void SetPalettes(u8 *bg, u8 *sp);
void ClearNametable();
u8 CharToTile(u8 c);
void AttrSetNow(u8 x, u8 y, u8 attr);
void AttrSetBlit(u8 x, u8 y, u8 attr);
void PutTileNow(u8 x, u8 y, u8 tile);
void PutTileBlit(u8 x, u8 y, u8 tile);
u16 NameAddr(u8 x, u8 y);
void PutStringNow(u8 x, u8 y, char *s);
void PutNumber2Now(u8 x, u8 y, u8 value);
void PutScoreNow(u8 x, u8 y);
void PutStringBlit(u8 x, u8 y, char *s);
void PutNumber2Blit(u8 x, u8 y, u8 value);
void PutScoreBlit(u8 x, u8 y);
void FillBlit(u8 x, u8 y, u8 count, u8 tile);
void DrawBoxNow(u8 x, u8 y, u8 w, u8 h);
void DrawBoxBlit(u8 x, u8 y, u8 w, u8 h);
void CatchUpAudioIfNeeded();

void Title_Start();
void Title_Update();
void Pause_Start();
void Pause_Update();
void GameOver_Start();
void GameOver_Update();
void Game_Start();
void Game_Update();

void Board_Clear();
u8 Board_Get(u8 x, u8 y);
void Board_Set(u8 x, u8 y, u8 v);
void SpawnMino();
void LockMino();
u8 CheckLines();
void LineClear_Start();
void LineClear_Update();
void LineClear_Finish();
u8 LineClear_IsClearedRow(u8 row);
u8 IsColliding(u8 x, u8 y, u8 rot, u8 mino);
void TryMoveLeft();
void TryMoveRight();
void TrySoftDrop();
void TryRotateCW();
void TryRotateCCW();
void TryHold();
void UpdateGravity();
void AddScoreForLines(u8 count);
u8 NextRandomMino();
void BagShuffle();
void BcdClear();
void BcdAddSmall(u8 amount);
void UpdateHudNumbers();

void Render_InitTitleScreen();
void Render_InitGameScreen();
void Render_DrawFrameNow();
void Render_DrawPanelsNow();
void Render_SetUiAttributesNow();
void Render_DrawBoardFullNow();
void Render_DrawBoardBlackBlit();
void Render_ClearFieldAttributeOwners();
void Render_RebuildFieldAttributes();
void Render_ResetFieldAttributesNow();
void Render_DrawBoardRowBlit(u8 y);
void Render_DrawBoardRowClearBlit(u8 y);
void Render_DrawBoardScreenAttrBlockBlit(u8 baseX, u8 baseY);
void Render_DrawLockedMinoBlocks(u16 shape);
void Render_UpdateActiveMinoSprites();
void Render_UpdatePreviewSprites();
void Render_UpdateHud();
void Render_UpdateHudNow();
void Title_DrawValues();
void Title_DrawCursor();

void Audio_Initialize();
void Audio_Update();
void Audio_PlayBgm(u8 id);
void Audio_StopBgm();
void Sfx_Play(u8 id);
void Music_Start(u8 id);
void Music_Stop();
void Music_Update();

define u8 BGM_NONE = 0;
define u8 BGM_TITLE = 1;
define u8 BGM_GAME = 2;
define u8 BGM_GAMEOVER = 3;

define u8 SFX_LOCK = 1;
define u8 SFX_HOLD = 2;
define u8 SFX_CLEAR = 3;

#endif
