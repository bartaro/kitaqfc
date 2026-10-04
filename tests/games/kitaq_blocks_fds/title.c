#include "kbfc.h"

u8 SettingStartLevel;
u8 SettingRandomizer;
u8 SettingNext;
u8 SettingHold;
u8 SettingMusic;
u8 SettingSound;
u8 TitleCursor;
u8 GameOverTimer;
u8 TitlePressStartVisible;
u8 PausePressStartVisible;
u8 GameOverPressStartVisible;

u8 Ui_BlinkVisible()
{
    if (NmiFrameCounter & 0x40) return FALSE;
    return TRUE;
}

void Ui_HideActiveSprites()
{
    for (u8 i = 0; i < 4; ++i)
    {
        __sprite_hide(i);
    }
}

void Ui_HidePreviewSprites()
{
    for (u8 i = 8; i < 16; ++i)
    {
        __sprite_hide(i);
    }
}

void Ui_DrawPressStartBlit(u8 x, u8 y, u8 visible)
{
    if (visible) PutStringBlit(x, y, "PRESS START");
    else FillBlit(x, y, 11, TILE_EMPTY);
}

void Ui_ClearPanelInnerBlit(u8 y)
{
    for (u8 row = 0; row < 3; ++row)
    {
        FillBlit(PANEL_X + 1, y + row, 11, TILE_EMPTY);
        FlushBlitChunk();
    }
}

void Ui_DrawModePanelsBlit(char *label, u8 labelX, u8 pressVisible)
{
    Ui_ClearPanelInnerBlit(PANEL_Y_NEXT);
    Ui_ClearPanelInnerBlit(PANEL_Y_HOLD);
    PutStringBlit(labelX, PANEL_Y_NEXT, label);
    FlushBlitChunk();
    Ui_DrawPressStartBlit(PANEL_X + 1, PANEL_Y_HOLD, pressVisible);
    FlushBlitChunk();
}

void Title_UpdatePressStartBlink()
{
    u8 visible = Ui_BlinkVisible();

    if (visible == TitlePressStartVisible) return;

    ClearBlitBuffer();
    Ui_DrawPressStartBlit(10, 24, visible);
    TitlePressStartVisible = visible;
}

void Pause_UpdatePressStartBlink()
{
    u8 visible = Ui_BlinkVisible();

    if (visible == PausePressStartVisible) return;

    ClearBlitBuffer();
    Ui_DrawPressStartBlit(PANEL_X + 1, PANEL_Y_HOLD, visible);
    PausePressStartVisible = visible;
}

void GameOver_UpdatePressStartBlink()
{
    u8 visible = Ui_BlinkVisible();

    if (visible == GameOverPressStartVisible) return;

    ClearBlitBuffer();
    Ui_DrawPressStartBlit(PANEL_X + 1, PANEL_Y_HOLD, visible);
    GameOverPressStartVisible = visible;
}

void Title_RedrawStable()
{
    ClearBlitBuffer();
    DisablePPU();
    Render_InitTitleScreen();
    TitlePressStartVisible = TRUE;
    EnablePPU();
}

void Title_Start()
{
    CurrentMode = MODE_TITLE;
    TitleCursor = 0;
    HideAllSprites();
    Title_RedrawStable();
    Audio_StopBgm();
}

void Title_DrawCursor()
{
    for (u8 i = 0; i < 6; ++i)
    {
        PutTileNow(5, 9 + i * 2, TILE_EMPTY);
    }
    PutTileNow(5, 9 + TitleCursor * 2, TILE_STAR);
}

void Title_DrawValues()
{
    PutNumber2Now(22, 9, SettingStartLevel);

    if (SettingRandomizer == RANDOMIZER_BAG) PutStringNow(20, 11, " BAG ");
    else PutStringNow(20, 11, "RETRO");

    if (SettingNext) PutStringNow(22, 13, "ON ");
    else PutStringNow(22, 13, "OFF");

    if (SettingHold) PutStringNow(22, 15, "ON ");
    else PutStringNow(22, 15, "OFF");

    if (SettingMusic) PutStringNow(22, 17, "ON ");
    else PutStringNow(22, 17, "OFF");

    if (SettingSound) PutStringNow(22, 19, "ON ");
    else PutStringNow(22, 19, "OFF");
}

void Title_DrawOnOffBlit(u8 y, u8 value)
{
    if (value) PutStringBlit(22, y, "ON ");
    else PutStringBlit(22, y, "OFF");
}

void Title_DrawValueBlit(u8 cursor)
{
    if (cursor == 0)
    {
        PutNumber2Blit(22, 9, SettingStartLevel);
    }
    else if (cursor == 1)
    {
        if (SettingRandomizer == RANDOMIZER_BAG) PutStringBlit(20, 11, " BAG ");
        else PutStringBlit(20, 11, "RETRO");
    }
    else if (cursor == 2)
    {
        Title_DrawOnOffBlit(13, SettingNext);
    }
    else if (cursor == 3)
    {
        Title_DrawOnOffBlit(15, SettingHold);
    }
    else if (cursor == 4)
    {
        Title_DrawOnOffBlit(17, SettingMusic);
    }
    else if (cursor == 5)
    {
        Title_DrawOnOffBlit(19, SettingSound);
    }
}

void Title_DrawCursorMoveBlit(u8 oldCursor)
{
    if (oldCursor != TitleCursor)
    {
        PutTileBlit(5, 9 + oldCursor * 2, TILE_EMPTY);
    }

    PutTileBlit(5, 9 + TitleCursor * 2, TILE_STAR);
}

void Title_Update()
{
    u8 oldCursor = TitleCursor;
    u8 cursorChanged = FALSE;
    u8 valueChanged = 0xFF;

    if (ButtonDown(BUTTON_START))
    {
        ClearBlitBuffer();
        Game_Start();
        return;
    }

    if (ButtonDown(BUTTON_UP))
    {
        if (TitleCursor == 0) TitleCursor = 5;
        else TitleCursor -= 1;
        cursorChanged = TRUE;
    }
    else if (ButtonDown(BUTTON_DOWN))
    {
        TitleCursor += 1;
        if (TitleCursor >= 6) TitleCursor = 0;
        cursorChanged = TRUE;
    }

    if (ButtonDown(BUTTON_LEFT) || ButtonDown(BUTTON_RIGHT) || ButtonDown(BUTTON_A))
    {
        if (TitleCursor == 0)
        {
            if (ButtonDown(BUTTON_LEFT))
            {
                if (SettingStartLevel) SettingStartLevel -= 1;
            }
            else
            {
                if (SettingStartLevel < 20) SettingStartLevel += 1;
            }
        }
        else if (TitleCursor == 1) SettingRandomizer = SettingRandomizer ^ 1;
        else if (TitleCursor == 2) SettingNext = SettingNext ^ 1;
        else if (TitleCursor == 3) SettingHold = SettingHold ^ 1;
        else if (TitleCursor == 4)
        {
            SettingMusic = SettingMusic ^ 1;
            Audio_StopBgm();
        }
        else if (TitleCursor == 5) SettingSound = SettingSound ^ 1;

        valueChanged = TitleCursor;
    }

    if (ButtonDown(BUTTON_SELECT))
    {
        SettingStartLevel = 0;
        SettingRandomizer = RANDOMIZER_BAG;
        SettingNext = TRUE;
        SettingHold = TRUE;
        SettingMusic = TRUE;
        SettingSound = TRUE;
        valueChanged = 6;
    }

    if (valueChanged != 0xFF)
    {
        if (valueChanged == 6)
        {
            Title_RedrawStable();
        }
        else
        {
            ClearBlitBuffer();
            Title_DrawValueBlit(valueChanged);
            if (cursorChanged) Title_DrawCursorMoveBlit(oldCursor);
        }
        return;
    }

    if (cursorChanged)
    {
        ClearBlitBuffer();
        Title_DrawCursorMoveBlit(oldCursor);
        return;
    }

    Title_UpdatePressStartBlink();
}

void Pause_Start()
{
    PreviousMode = MODE_GAME;
    CurrentMode = MODE_PAUSE;
    PausePressStartVisible = Ui_BlinkVisible();
    ClearBlitBuffer();
    Ui_HideActiveSprites();
    Ui_HidePreviewSprites();
    Render_DrawBoardBlackBlit();
    Ui_DrawModePanelsBlit("PAUSE", PANEL_X + 4, PausePressStartVisible);
}

void Pause_Update()
{
    if (ButtonDown(BUTTON_START))
    {
        CurrentMode = MODE_GAME;
        ClearBlitBuffer();
        DisablePPU();
        Render_InitGameScreen();
        Render_UpdateHudNow();
        Render_UpdateActiveMinoSprites();
        EnablePPU();
        return;
    }

    if (ButtonDown(BUTTON_SELECT))
    {
        Title_Start();
        return;
    }

    Pause_UpdatePressStartBlink();
}

void GameOver_Start()
{
    CurrentMode = MODE_GAMEOVER;
    GameOverTimer = 60;
    GameOverPressStartVisible = Ui_BlinkVisible();
    ClearBlitBuffer();
    Ui_HidePreviewSprites();
    Ui_DrawModePanelsBlit("GAME OVER", PANEL_X + 2, GameOverPressStartVisible);
    Audio_PlayBgm(BGM_GAMEOVER);
}

void GameOver_Update()
{
    GameOver_UpdatePressStartBlink();

    if (GameOverTimer)
    {
        GameOverTimer -= 1;
        return;
    }

    if (ButtonDown(BUTTON_START) || ButtonDown(BUTTON_A) || ButtonDown(BUTTON_B))
    {
        Title_Start();
    }
}
