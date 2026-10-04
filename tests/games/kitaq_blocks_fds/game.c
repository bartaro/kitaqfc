#include "kbfc.h"

u8 Board[BOARD_H * BOARD_STRIDE];
u8 CurrentMino;
u8 CurrentRot;
u8 CurrentX;
u8 CurrentY;
u8 NextMino;
u8 HoldMino;
u8 HoldUsed;
u8 GameOverFlag;
u8 Level;
u8 Lines;
u16 Score;
u8 ScoreBCD[4];
u8 FrameCounter;
u8 HudDirty;
u8 RotateCooldown;
u8 DropBonus;
u8 LineClearActive;
u8 LineClearPhase;
u8 LineClearWait;
u8 LineClearCount;
u8 LineClearDirtyBottom;
u8 LineClearRedrawRow;
u8 LineClearRows[4];

u8 GravityCounter;
u8 LockCounter;
u8 LockResetCount;
u8 DasLeft;
u8 DasRight;
u8 Bag[7];
u8 BagIndex;
u8 Hist[4];

void Game_Start()
{
    CurrentMode = MODE_GAME;
    HideAllSprites();
    ClearBlitBuffer();
    DisablePPU();

    Board_Clear();
    Render_ClearFieldAttributeOwners();
    BcdClear();
    Score = 0;
    Lines = 0;
    Level = SettingStartLevel;
    HoldMino = MINO_NONE;
    HoldUsed = FALSE;
    GameOverFlag = FALSE;
    HudDirty = FALSE;
    RotateCooldown = 0;
    DropBonus = 0;
    LineClearActive = FALSE;
    LineClearPhase = 0;
    LineClearWait = 0;
    LineClearCount = 0;
    LineClearDirtyBottom = 0;
    BagIndex = 7;
    Hist[0] = 0;
    Hist[1] = 1;
    Hist[2] = 2;
    Hist[3] = 3;

    NextMino = NextRandomMino();
    SpawnMino();

    ClearBlitBuffer();
    Render_InitGameScreen();
    UpdateHudNumbers();
    Render_UpdateHudNow();
    Render_UpdateActiveMinoSprites();
    HudDirty = FALSE;
    EnablePPU();

    if (SettingMusic) Audio_PlayBgm(BGM_GAME);
}

void Game_Update()
{
    FrameCounter += 1;

    if (LineClearActive)
    {
        LineClear_Update();
        if (CurrentMode == MODE_GAME)
        {
            Render_UpdatePreviewSprites();
        }
        return;
    }

    if (ButtonDown(BUTTON_START))
    {
        Pause_Start();
        return;
    }

    if (RotateCooldown)
    {
        RotateCooldown -= 1;
    }

    if (RotateCooldown == 0)
    {
        if (ButtonDown(BUTTON_A))
        {
            TryRotateCW();
            RotateCooldown = 8;
        }
        else if (ButtonDown(BUTTON_B))
        {
            TryRotateCCW();
            RotateCooldown = 8;
        }
    }

    if (ButtonDown(BUTTON_UP)) TryHold();

    if (ButtonDown(BUTTON_LEFT))
    {
        TryMoveLeft();
        DasLeft = 10;
    }
    else if (Button(BUTTON_LEFT))
    {
        if (DasLeft) DasLeft -= 1;
        else
        {
            TryMoveLeft();
            DasLeft = 2;
        }
    }
    else
    {
        DasLeft = 0;
    }

    if (ButtonDown(BUTTON_RIGHT))
    {
        TryMoveRight();
        DasRight = 10;
    }
    else if (Button(BUTTON_RIGHT))
    {
        if (DasRight) DasRight -= 1;
        else
        {
            TryMoveRight();
            DasRight = 2;
        }
    }
    else
    {
        DasRight = 0;
    }

    if (Button(BUTTON_DOWN))
    {
        TrySoftDrop();
    }

    UpdateGravity();
    if (LineClearActive) return;
    if (CurrentMode != MODE_GAME || GameOverFlag) return;

    Render_UpdateActiveMinoSprites();
    Render_UpdatePreviewSprites();

    if (HudDirty)
    {
        Render_UpdateHud();
        HudDirty = FALSE;
    }
}

#pragma fixed_bank 0
void Board_Clear()
{
    for (u8 y = 0; y < BOARD_H; ++y)
    {
        for (u8 x = 0; x < FIELD_W; ++x)
        {
            Board_Set(x, y, 0);
        }
    }
}

u8 Board_Get(u8 x, u8 y)
{
    u16 index;

    if (x >= FIELD_W) return 0;
    if (y >= BOARD_H) return 0;

    index = y;
    index = index << 4;
    return Board[index + x];
}

void Board_Set(u8 x, u8 y, u8 v)
{
    u16 index;

    if (x >= FIELD_W) return;
    if (y >= BOARD_H) return;

    index = y;
    index = index << 4;
    Board[index + x] = v;
}

u8 IsColliding(u8 x, u8 y, u8 rot, u8 mino)
{
    u16 shape = mino * 64 + rot * 16;

    for (u8 cy = 0; cy < 4; ++cy)
    {
        for (u8 cx = 0; cx < 4; ++cx)
        {
            if (MinoShapes[shape + cy * 4 + cx])
            {
                u8 bx = x + cx;
                u8 by = y + cy;

                if (bx >= FIELD_W) return TRUE;
                if (by >= BOARD_H) return TRUE;
                if (Board_Get(bx, by)) return TRUE;
            }
        }
    }

    return FALSE;
}

void SpawnMino()
{
    CurrentMino = NextMino;
    NextMino = NextRandomMino();
    CurrentRot = 0;
    CurrentX = 3;

    // The board keeps two hidden rows, so y=2 is the first visible row.
    CurrentY = 2;
    HoldUsed = FALSE;
    GravityCounter = GravityFrames[Level];
    LockCounter = LOCK_FRAMES;
    LockResetCount = 0;
    RotateCooldown = 0;
    DropBonus = 0;

    if (IsColliding(CurrentX, CurrentY, CurrentRot, CurrentMino))
    {
        GameOverFlag = TRUE;
        Render_UpdateActiveMinoSprites();
        GameOver_Start();
        return;
    }

    Render_UpdatePreviewSprites();
}

#pragma fixed_bank -1
void LockMino()
{
    u16 shape = CurrentMino * 64 + CurrentRot * 16;

    for (u8 i = 0; i < 4; ++i)
    {
        __sprite_hide(i);
    }

    for (u8 cy = 0; cy < 4; ++cy)
    {
        for (u8 cx = 0; cx < 4; ++cx)
        {
            if (MinoShapes[shape + cy * 4 + cx])
            {
                u8 bx = CurrentX + cx;
                u8 by = CurrentY + cy;

                if (bx < FIELD_W && by < BOARD_H)
                {
                    Board_Set(bx, by, CurrentMino + 1);
                }
            }
        }
    }

    Render_DrawLockedMinoBlocks(shape);

    if (DropBonus)
    {
        BcdAddSmall(DropBonus);
        DropBonus = 0;
        UpdateHudNumbers();
        HudDirty = TRUE;
    }

    Sfx_Play(SFX_LOCK);
    if (CheckLines()) return;
    SpawnMino();
}

#pragma fixed_bank 0
u8 CheckLines()
{
    u8 y = BOARD_H - 1;

    LineClearCount = 0;
    LineClearDirtyBottom = 0;

    while (TRUE)
    {
        u8 full = TRUE;

        for (u8 x = 0; x < FIELD_W; ++x)
        {
            if (Board_Get(x, y) == 0) full = FALSE;
        }

        if (full)
        {
            if (LineClearCount < 4)
            {
                LineClearRows[LineClearCount] = y;
                LineClearCount += 1;
            }
            if (LineClearDirtyBottom < y) LineClearDirtyBottom = y;
        }

        if (y == 2) break;
        y -= 1;
    }

    if (LineClearCount == 0) return FALSE;

    for (u8 a = 0; a < LineClearCount; ++a)
    {
        for (u8 b = a + 1; b < LineClearCount; ++b)
        {
            if (LineClearRows[a] > LineClearRows[b])
            {
                u8 t = LineClearRows[a];
                LineClearRows[a] = LineClearRows[b];
                LineClearRows[b] = t;
            }
        }
    }

    LineClear_Start();
    return TRUE;
}
#pragma fixed_bank -1

void LineClear_Start()
{
    LineClearActive = TRUE;
    LineClearPhase = 0;
    LineClearWait = 0;
    LineClearRedrawRow = 2;

    for (u8 i = 0; i < 4; ++i)
    {
        __sprite_hide(i);
    }

    Sfx_Play(SFX_CLEAR);
}

void LineClear_DrawClearedRowsBlank()
{
    for (u8 i = 0; i < LineClearCount; ++i)
    {
        Render_DrawBoardRowClearBlit(LineClearRows[i]);
        FlushBlitChunk();
    }
}

void LineClear_Update()
{
    u8 newLevel;

    if (LineClearWait)
    {
        LineClearWait -= 1;
        return;
    }

    if (LineClearPhase == 0)
    {
        LineClear_DrawClearedRowsBlank();
        LineClearPhase = 1;
        LineClearWait = 8;
        return;
    }

    if (LineClearPhase == 1)
    {
        LineClear_Finish();
        Render_RebuildFieldAttributes();
        LineClearRedrawRow = LineClearDirtyBottom;
        LineClearPhase = 2;
        return;
    }

    Render_DrawBoardRowBlit(LineClearRedrawRow);
    if (LineClearRedrawRow <= 2)
    {
        LineClearActive = FALSE;
        AddScoreForLines(LineClearCount);
        Lines += LineClearCount;
        if (Lines >= 100) Lines = 99;

        SplitDecimal10(Lines);
        newLevel = SettingStartLevel + DecimalTens;
        if (newLevel > 20) newLevel = 20;
        if (newLevel != Level)
        {
            Level = newLevel;
        }

        UpdateHudNumbers();
        HudDirty = TRUE;
        LineClearCount = 0;
        SpawnMino();
    }
    else
    {
        LineClearRedrawRow -= 1;
    }
}

#pragma fixed_bank 0
u8 LineClear_IsClearedRow(u8 row)
{
    for (u8 i = 0; i < LineClearCount; ++i)
    {
        if (LineClearRows[i] == row) return TRUE;
    }

    return FALSE;
}

void LineClear_Finish()
{
    u8 collapsed = LineClearCount;
    u8 src;
    u8 dst;

    if (collapsed == 0) return;

    src = LineClearDirtyBottom;
    dst = LineClearDirtyBottom;

    while (TRUE)
    {
        if (!LineClear_IsClearedRow(src))
        {
            for (u8 x = 0; x < FIELD_W; ++x)
            {
                Board_Set(x, dst, Board_Get(x, src));
            }

            if (dst) dst -= 1;
        }

        if (src == 2) break;
        src -= 1;
    }

    while (TRUE)
    {
        for (u8 clearX = 0; clearX < FIELD_W; ++clearX)
        {
            Board_Set(clearX, dst, 0);
        }

        if (dst == 2) break;
        dst -= 1;
    }

    if (LineClearDirtyBottom < 2) LineClearDirtyBottom = 2;
}

#pragma fixed_bank 0
void AddLineScoreOnce(u8 count)
{
    if (count == 1)
    {
        BcdAddSmall(40);
    }
    else if (count == 2)
    {
        BcdAddSmall(100);
    }
    else if (count == 3)
    {
        for (u8 i = 0; i < 3; ++i)
        {
            BcdAddSmall(100);
        }
    }
    else
    {
        for (u8 j = 0; j < 12; ++j)
        {
            BcdAddSmall(100);
        }
    }
}

#pragma fixed_bank -1

void TryMoveLeft()
{
    if (CurrentX > 0 && !IsColliding(CurrentX - 1, CurrentY, CurrentRot, CurrentMino))
    {
        CurrentX -= 1;
        if (LockResetCount < 15)
        {
            LockCounter = LOCK_FRAMES;
            LockResetCount += 1;
        }
    }
}

void TryMoveRight()
{
    if (!IsColliding(CurrentX + 1, CurrentY, CurrentRot, CurrentMino))
    {
        CurrentX += 1;
        if (LockResetCount < 15)
        {
            LockCounter = LOCK_FRAMES;
            LockResetCount += 1;
        }
    }
}

void TrySoftDrop()
{
    if (!IsColliding(CurrentX, CurrentY + 1, CurrentRot, CurrentMino))
    {
        CurrentY += 1;
        if (DropBonus < 99) DropBonus += 1;
        LockCounter = LOCK_FRAMES;
    }
    else
    {
        if (LockCounter > SOFTLOCK_FRAMES) LockCounter = SOFTLOCK_FRAMES;
    }
}

void TryRotateCW()
{
    u8 r = CurrentRot + 1;
    if (r >= 4) r = 0;

    if (!IsColliding(CurrentX, CurrentY, r, CurrentMino))
    {
        CurrentRot = r;
        LockCounter = LOCK_FRAMES;
        return;
    }

    if (!IsColliding(CurrentX + 1, CurrentY, r, CurrentMino))
    {
        CurrentX += 1;
        CurrentRot = r;
        LockCounter = LOCK_FRAMES;
        return;
    }

    if (CurrentX > 0 && !IsColliding(CurrentX - 1, CurrentY, r, CurrentMino))
    {
        CurrentX -= 1;
        CurrentRot = r;
        LockCounter = LOCK_FRAMES;
    }
}

void TryRotateCCW()
{
    u8 r;

    if (CurrentRot == 0) r = 3;
    else r = CurrentRot - 1;

    if (!IsColliding(CurrentX, CurrentY, r, CurrentMino))
    {
        CurrentRot = r;
        LockCounter = LOCK_FRAMES;
        return;
    }

    if (!IsColliding(CurrentX + 1, CurrentY, r, CurrentMino))
    {
        CurrentX += 1;
        CurrentRot = r;
        LockCounter = LOCK_FRAMES;
        return;
    }

    if (CurrentX > 0 && !IsColliding(CurrentX - 1, CurrentY, r, CurrentMino))
    {
        CurrentX -= 1;
        CurrentRot = r;
        LockCounter = LOCK_FRAMES;
    }
}

void TryHold()
{
    if (!SettingHold || HoldUsed) return;

    u8 old = HoldMino;

    if (old == MINO_NONE)
    {
        HoldMino = CurrentMino;
        CurrentMino = NextMino;
        NextMino = NextRandomMino();
    }
    else
    {
        HoldMino = CurrentMino;
        CurrentMino = old;
    }

    CurrentRot = 0;
    CurrentX = 3;
    CurrentY = 2;
    DropBonus = 0;
    GravityCounter = GravityFrames[Level];
    LockCounter = LOCK_FRAMES;
    LockResetCount = 0;
    HoldUsed = TRUE;

    if (IsColliding(CurrentX, CurrentY, CurrentRot, CurrentMino))
    {
        GameOverFlag = TRUE;
        Render_UpdateActiveMinoSprites();
        GameOver_Start();
        return;
    }

    Render_UpdatePreviewSprites();
    Sfx_Play(SFX_HOLD);
}

void UpdateGravity()
{
    if (GameOverFlag) return;

    if (IsColliding(CurrentX, CurrentY + 1, CurrentRot, CurrentMino))
    {
        if (LockCounter) LockCounter -= 1;
        else LockMino();
        return;
    }

    LockCounter = LOCK_FRAMES;

    if (GravityCounter)
    {
        GravityCounter -= 1;
    }
    else
    {
        GravityCounter = GravityFrames[Level];
        CurrentY += 1;
    }
}

void AddScoreForLines(u8 count)
{
    for (u8 i = 0; i <= Level; ++i)
    {
        AddLineScoreOnce(count);
    }
}

void BcdClear()
{
    for (u8 i = 0; i < 4; ++i)
    {
        ScoreBCD[i] = 0;
    }
}

void BcdAddSmall(u8 amount)
{
    ScoreBCD[0] += amount;

    while (ScoreBCD[0] >= 10)
    {
        ScoreBCD[0] -= 10;
        ScoreBCD[1] += 1;
    }

    while (ScoreBCD[1] >= 100)
    {
        ScoreBCD[1] -= 100;
        ScoreBCD[2] += 1;
    }

    while (ScoreBCD[2] >= 100)
    {
        ScoreBCD[2] -= 100;
        ScoreBCD[3] += 1;
    }

    if (ScoreBCD[3] >= 100) ScoreBCD[3] = 99;
}

void UpdateHudNumbers()
{
}

void BagShuffle()
{
    for (u8 i = 0; i < 7; ++i)
    {
        Bag[i] = i;
    }

    for (u8 j = 0; j < 7; ++j)
    {
        u8 r = GetRandomByte();

        while (r >= 7)
        {
            r -= 7;
        }

        u8 tmp = Bag[j];
        Bag[j] = Bag[r];
        Bag[r] = tmp;
    }

    BagIndex = 0;
}

u8 NextRandomMino()
{
    if (SettingRandomizer == RANDOMIZER_BAG)
    {
        if (BagIndex >= 7) BagShuffle();

        u8 v = Bag[BagIndex];
        BagIndex += 1;
        return v;
    }

    u8 cand = 0;

    for (u8 tries = 0; tries < 6; ++tries)
    {
        cand = GetRandomByte();

        while (cand >= 7)
        {
            cand -= 7;
        }

        u8 hit = FALSE;
        for (u8 i = 0; i < 4; ++i)
        {
            if (Hist[i] == cand) hit = TRUE;
        }

        if (!hit) break;
    }

    Hist[3] = Hist[2];
    Hist[2] = Hist[1];
    Hist[1] = Hist[0];
    Hist[0] = cand;
    return cand;
}
