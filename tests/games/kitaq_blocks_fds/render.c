#include "kbfc.h"

u8 Render_FieldAttributeForScreen(u8 screenX, u8 screenY);
u8 Render_FieldAttrIndexFromBase(u8 baseX, u8 baseY);
void Render_ClaimBoardAttrForCell(u8 x, u8 y, u8 cell);

u8 FieldAttrOwners[FIELD_ATTR_W * FIELD_ATTR_H];

u8 Render_CellTile(u8 cell)
{
    if (cell == 0) return TILE_EMPTY;
    return TILE_BLOCK_L;
}

u8 Render_FieldAttrIndexFromBase(u8 baseX, u8 baseY)
{
    if (baseX < FIELD_X) return FIELD_ATTR_EMPTY;
    if (baseX >= FIELD_X + FIELD_W) return FIELD_ATTR_EMPTY;
    if (baseY < FIELD_Y) return FIELD_ATTR_EMPTY;
    if (baseY >= FIELD_Y + FIELD_H) return FIELD_ATTR_EMPTY;

    return ((baseY - FIELD_Y) >> 1) * FIELD_ATTR_W + ((baseX - FIELD_X) >> 1);
}

u8 Render_BlockHasCells(u8 baseX, u8 baseY)
{
    for (u8 yy = 0; yy < 2; ++yy)
    {
        u8 sy = baseY + yy;

        if (sy >= FIELD_Y && sy < FIELD_Y + FIELD_H)
        {
            for (u8 xx = 0; xx < 2; ++xx)
            {
                u8 sx = baseX + xx;

                if (sx >= FIELD_X && sx < FIELD_X + FIELD_W)
                {
                    if (Board_Get(sx - FIELD_X, sy - FIELD_Y + 2)) return TRUE;
                }
            }
        }
    }

    return FALSE;
}

#pragma fixed_bank -1

void Render_InitTitleScreen()
{
    ClearNametable();
    SetPalettes(PaletteBg, PaletteSp);
    Render_SetUiAttributesNow();
    PutStringNow(6, 3, "KITAQ BLOCKS FC");
    DrawBoxNow(4, 7, 24, 15);
    PutStringNow(7, 9, "START LEVEL");
    PutStringNow(7, 11, "RANDOMIZER");
    PutStringNow(7, 13, "NEXT");
    PutStringNow(7, 15, "HOLD");
    PutStringNow(7, 17, "MUSIC");
    PutStringNow(7, 19, "SOUND");
    PutStringNow(10, 24, "PRESS START");
    Title_DrawValues();
    Title_DrawCursor();
}

void Render_InitGameScreen()
{
    ClearNametable();
    SetPalettes(PaletteBg, PaletteSp);
    Render_SetUiAttributesNow();
    Render_RebuildFieldAttributes();
    Render_DrawFrameNow();
    Render_DrawPanelsNow();
    Render_ResetFieldAttributesNow();
    Render_DrawBoardFullNow();
    PutStringNow(1, 28, "A/B:ROTATE UP:HOLD START:PAUSE");
}

void Render_SetUiAttributesNow()
{
    for (u8 y = 0; y < 30; y += 2)
    {
        for (u8 x = 0; x < 32; x += 2)
        {
            AttrSetNow(x, y, BG_ATTR_UI);
        }
    }
}

void Render_DrawFrameNow()
{
    for (u8 y = 0; y < FIELD_H; ++y)
    {
        PutTileNow(FIELD_X - 1, FIELD_Y + y, TILE_FRAME);

        for (u8 x = 0; x < FIELD_W; ++x)
        {
            PutTileNow(FIELD_X + x, FIELD_Y + y, TILE_GRID);
        }

        PutTileNow(FIELD_X + FIELD_W, FIELD_Y + y, TILE_FRAME);
    }

    for (u8 bx = 0; bx < FIELD_W + 2; ++bx)
    {
        PutTileNow(FIELD_X - 1 + bx, FIELD_Y + FIELD_H, TILE_FRAME);
    }
}

void Render_DrawPanelsNow()
{
    DrawBoxNow(PANEL_X, PANEL_Y_NEXT - 1, 13, 5);
    PutStringNow(PANEL_X + 4, PANEL_Y_NEXT, "NEXT");

    DrawBoxNow(PANEL_X, PANEL_Y_HOLD - 1, 13, 5);
    PutStringNow(PANEL_X + 4, PANEL_Y_HOLD, "HOLD");

    DrawBoxNow(PANEL_X, PANEL_Y_SCORE - 1, 13, 4);
    PutStringNow(PANEL_X + 3, PANEL_Y_SCORE, "SCORE");

    DrawBoxNow(PANEL_X, PANEL_Y_LEVEL - 1, 13, 4);
    PutStringNow(PANEL_X + 3, PANEL_Y_LEVEL, "LEVEL");

    DrawBoxNow(PANEL_X, PANEL_Y_LINES - 1, 13, 4);
    PutStringNow(PANEL_X + 3, PANEL_Y_LINES, "LINES");
}

void Render_DrawBoardFullNow()
{
    for (u8 y = 2; y < BOARD_H; ++y)
    {
        for (u8 x = 0; x < FIELD_W; ++x)
        {
            u8 sx = FIELD_X + x;
            u8 sy = FIELD_Y + y - 2;
            u8 cell = Board_Get(x, y);
            u8 tile = Render_CellTile(cell);

            PutTileNow(sx, sy, tile);
        }
    }
}

void Render_DrawBoardBlackBlit()
{
    for (u8 y = 0; y < FIELD_H; ++y)
    {
        FillBlit(FIELD_X, FIELD_Y + y, FIELD_W, TILE_EMPTY);
        FlushBlitChunk();
    }
}

#pragma fixed_bank -1
u8 Render_FieldAttributeForScreen(u8 screenX, u8 screenY)
{
    u8 baseX = screenX & 0xFE;
    u8 baseY = screenY & 0xFE;
    u8 index = Render_FieldAttrIndexFromBase(baseX, baseY);
    u8 attr;

    if (index == FIELD_ATTR_EMPTY) return BG_ATTR_UI;

    attr = FieldAttrOwners[index];
    if (attr == FIELD_ATTR_EMPTY) return BG_ATTR_UI;
    return attr;
}

#pragma fixed_bank -1
void Render_ResetFieldAttributesNow()
{
    for (u8 y = FIELD_Y; y < FIELD_Y + FIELD_H; y += 2)
    {
        for (u8 x = FIELD_X; x < FIELD_X + FIELD_W; x += 2)
        {
            AttrSetNow(x, y, Render_FieldAttributeForScreen(x, y));
        }
    }
}

void Render_ClearFieldAttributeOwners()
{
    for (u8 i = 0; i < FIELD_ATTR_W * FIELD_ATTR_H; ++i)
    {
        FieldAttrOwners[i] = FIELD_ATTR_EMPTY;
    }
}

void Render_RebuildFieldAttributes()
{
    for (u8 ay = 0; ay < FIELD_ATTR_H; ++ay)
    {
        for (u8 ax = 0; ax < FIELD_ATTR_W; ++ax)
        {
            u8 baseX = FIELD_X + ax * 2;
            u8 baseY = FIELD_Y + ay * 2;
            u8 index = ay * FIELD_ATTR_W + ax;

            if (Render_BlockHasCells(baseX, baseY))
            {
                FieldAttrOwners[index] = BG_ATTR_UI;
            }
            else
            {
                FieldAttrOwners[index] = FIELD_ATTR_EMPTY;
            }
        }
    }
}

void Render_ClaimBoardAttrForCell(u8 x, u8 y, u8 cell)
{
    u8 baseX;
    u8 baseY;
    u8 index;

    if (x >= FIELD_W) return;
    if (y < 2) return;
    if (y >= BOARD_H) return;

    baseX = (FIELD_X + x) & 0xFE;
    baseY = (FIELD_Y + y - 2) & 0xFE;
    index = Render_FieldAttrIndexFromBase(baseX, baseY);
    if (index == FIELD_ATTR_EMPTY) return;

    if (cell) FieldAttrOwners[index] = BG_ATTR_UI;
    else FieldAttrOwners[index] = FIELD_ATTR_EMPTY;
}

void Render_DrawBoardScreenAttrBlockBlit(u8 baseX, u8 baseY)
{
    u8 attr;

    attr = Render_FieldAttributeForScreen(baseX, baseY);
    AttrSetBlit(baseX, baseY, attr);

    for (u8 yy = 0; yy < 2; ++yy)
    {
        u8 sy = baseY + yy;
        if (sy >= FIELD_Y && sy < FIELD_Y + FIELD_H)
        {
            for (u8 xx = 0; xx < 2; ++xx)
            {
                u8 sx = baseX + xx;
                if (sx >= FIELD_X && sx < FIELD_X + FIELD_W)
                {
                    u8 bx = sx - FIELD_X;
                    u8 by = sy - FIELD_Y + 2;
                    u8 cell = Board_Get(bx, by);
                    u8 tile = Render_CellTile(cell);

                    PutTileBlit(sx, sy, tile);
                }
            }
        }
    }
}

void Render_DrawLockedMinoBlocks(u16 shape)
{
    u8 drawCount = 0;
    u8 drawX0 = 0xFF;
    u8 drawY0 = 0xFF;
    u8 drawX1 = 0xFF;
    u8 drawY1 = 0xFF;
    u8 drawX2 = 0xFF;
    u8 drawY2 = 0xFF;
    u8 drawX3 = 0xFF;
    u8 drawY3 = 0xFF;

    for (u8 cy = 0; cy < 4; ++cy)
    {
        for (u8 cx = 0; cx < 4; ++cx)
        {
            if (MinoShapes[shape + cy * 4 + cx])
            {
                u8 bx = CurrentX + cx;
                u8 by = CurrentY + cy;

                if (bx < FIELD_W && by >= 2 && by < BOARD_H)
                {
                    u8 baseX = (FIELD_X + bx) & 0xFE;
                    u8 baseY = (FIELD_Y + by - 2) & 0xFE;
                    u8 duplicate = FALSE;

                    Render_ClaimBoardAttrForCell(bx, by, CurrentMino + 1);

                    if (drawCount > 0 && drawX0 == baseX && drawY0 == baseY) duplicate = TRUE;
                    if (drawCount > 1 && drawX1 == baseX && drawY1 == baseY) duplicate = TRUE;
                    if (drawCount > 2 && drawX2 == baseX && drawY2 == baseY) duplicate = TRUE;
                    if (drawCount > 3 && drawX3 == baseX && drawY3 == baseY) duplicate = TRUE;

                    if (!duplicate)
                    {
                        if (drawCount == 0)
                        {
                            drawX0 = baseX;
                            drawY0 = baseY;
                        }
                        else if (drawCount == 1)
                        {
                            drawX1 = baseX;
                            drawY1 = baseY;
                        }
                        else if (drawCount == 2)
                        {
                            drawX2 = baseX;
                            drawY2 = baseY;
                        }
                        else if (drawCount == 3)
                        {
                            drawX3 = baseX;
                            drawY3 = baseY;
                        }

                        drawCount += 1;
                    }
                }
            }
        }
    }

    if (drawCount > 0)
    {
        Render_DrawBoardScreenAttrBlockBlit(drawX0, drawY0);
        FlushBlitChunk();
    }
    if (drawCount > 1)
    {
        Render_DrawBoardScreenAttrBlockBlit(drawX1, drawY1);
        FlushBlitChunk();
    }
    if (drawCount > 2)
    {
        Render_DrawBoardScreenAttrBlockBlit(drawX2, drawY2);
        FlushBlitChunk();
    }
    if (drawCount > 3)
    {
        Render_DrawBoardScreenAttrBlockBlit(drawX3, drawY3);
        FlushBlitChunk();
    }
}

void Render_DrawBoardRowBlit(u8 y)
{
    u8 sy;
    u16 addr;

    if (y < 2) return;
    if (y >= BOARD_H) return;

    sy = FIELD_Y + y - 2;

    for (u8 ax = 0; ax < FIELD_W; ax += 2)
    {
        AttrSetBlit(FIELD_X + ax, sy, Render_FieldAttributeForScreen(FIELD_X + ax, sy));
    }
    FlushBlitChunk();

    addr = NameAddr(FIELD_X, sy);
    for (u8 x = 0; x < FIELD_W; ++x)
    {
        u8 cell = Board_Get(x, y);
        u8 tile = Render_CellTile(cell);

        __vramq_put(addr + x, tile);
        if (x == 4 || x == FIELD_W - 1)
        {
            FlushBlitChunk();
        }
    }
}

void Render_DrawBoardRowClearBlit(u8 y)
{
    u8 sy;

    if (y < 2) return;
    if (y >= BOARD_H) return;

    sy = FIELD_Y + y - 2;
    FillBlit(FIELD_X, sy, FIELD_W, TILE_EMPTY);
}

#pragma fixed_bank -1
void Render_UpdateActiveMinoSprites()
{
    u16 shape = CurrentMino * 64 + CurrentRot * 16;
    u8 si = 0;
    u8 tile = MinoTile[CurrentMino];
    u8 attr = MinoSpriteAttr[CurrentMino];

    for (u8 cy = 0; cy < 4; ++cy)
    {
        for (u8 cx = 0; cx < 4; ++cx)
        {
            if (MinoShapes[shape + cy * 4 + cx])
            {
                if (si < 4)
                {
                    u8 sx = CurrentX + cx;
                    u8 sy = CurrentY + cy;

                    if (sx < FIELD_W && sy >= 2 && sy < BOARD_H)
                    {
                        __sprite_set(
                            si,
                            (FIELD_X + sx) * 8,
                            (FIELD_Y + sy - 2) * 8 - 1,
                            tile,
                            attr
                        );
                    }
                    else
                    {
                        __sprite_hide(si);
                    }

                    si += 1;
                }
            }
        }
    }

    while (si < 4)
    {
        __sprite_hide(si);
        si += 1;
    }
}

void DrawMiniMino(u8 base, u8 mino, u8 tx, u8 ty)
{
    for (u8 i = 0; i < 4; ++i)
    {
        __sprite_hide(base + i);
    }

    if (mino == MINO_NONE) return;

    u16 shape = mino * 64;
    u8 si = 0;
    u8 tile = MinoTile[mino];
    u8 attr = MinoSpriteAttr[mino];

    for (u8 cy = 0; cy < 4; ++cy)
    {
        for (u8 cx = 0; cx < 4; ++cx)
        {
            if (MinoShapes[shape + cy * 4 + cx] && si < 4)
            {
                __sprite_set(
                    base + si,
                    (tx + cx) * 8,
                    (ty + cy) * 8 - 1,
                    tile,
                    attr
                );
                si += 1;
            }
        }
    }
}

void Render_UpdatePreviewSprites()
{
    if (SettingNext) DrawMiniMino(8, NextMino, PANEL_X + 4, PANEL_Y_NEXT + 1);
    else DrawMiniMino(8, MINO_NONE, PANEL_X + 4, PANEL_Y_NEXT + 1);

    if (SettingHold) DrawMiniMino(12, HoldMino, PANEL_X + 4, PANEL_Y_HOLD + 1);
    else DrawMiniMino(12, MINO_NONE, PANEL_X + 4, PANEL_Y_HOLD + 1);
}

#pragma fixed_bank -1
void Render_UpdateHud()
{
    PutScoreBlit(PANEL_X + 3, PANEL_Y_SCORE + 1);
    PutNumber2Blit(PANEL_X + 5, PANEL_Y_LEVEL + 1, Level);
    PutNumber2Blit(PANEL_X + 5, PANEL_Y_LINES + 1, Lines);
    Render_UpdatePreviewSprites();
}

void Render_UpdateHudNow()
{
    PutScoreNow(PANEL_X + 3, PANEL_Y_SCORE + 1);
    PutNumber2Now(PANEL_X + 5, PANEL_Y_LEVEL + 1, Level);
    PutNumber2Now(PANEL_X + 5, PANEL_Y_LINES + 1, Lines);
    Render_UpdatePreviewSprites();
}
