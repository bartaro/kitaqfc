u8 tick()
{
    u8 a;
    a = 3;
    while (a != 0)
    {
        a--;
    }

    if (a == 0)
    {
        return 1;
    }

    return 0;
}

u8 main()
{
    u8 x;
    x = tick();

    if (x != 0)
    {
        x = x + 1;
    }
    else
    {
        x = 0;
    }

    while (x < 5)
    {
        x++;
    }

    return x;
}
