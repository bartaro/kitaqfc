#ifndef CORE_H
#define CORE_H

// Public scalar aliases for the compiler's byte and 16-bit integer types.
typedef unsigned char u8;
typedef unsigned short u16;
typedef signed char s8;
typedef signed short s16;

// Boolean convenience values; API functions returning bit masks need not normalize to TRUE.
#define TRUE 1
#define FALSE 0

#endif
