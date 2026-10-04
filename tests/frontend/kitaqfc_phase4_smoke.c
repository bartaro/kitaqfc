unsigned short add16(unsigned short a, unsigned short b)
{
    unsigned short s;
    s = a + b;
    return s;
}

unsigned short clamp16(unsigned short x, unsigned short limit)
{
    if (x > limit) return limit;
    return x;
}

unsigned char is_zero16(unsigned short x)
{
    if (x == 0) return 1;
    return 0;
}

unsigned short step_counter(unsigned short x)
{
    unsigned short i;
    i = 0;
    while (i < 3)
    {
        x = x + 1;
        i = i + 1;
    }
    return x;
}

unsigned short main(void)
{
    unsigned short a;
    unsigned short b;
    unsigned short c;
    a = 0x1234;
    b = 0x0102;
    c = add16(a, b);
    c = clamp16(c, 0x1300);
    c = step_counter(c);
    if (is_zero16(c))
        c = 1;
    return c;
}
