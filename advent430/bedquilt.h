#ifndef BEDQUILT_H
#define BEDQUILT_H

typedef unsigned long size_t;
typedef long ssize_t;
typedef long ptrdiff_t;
typedef long intptr_t;
typedef unsigned long uintptr_t;
typedef float float_t;
typedef double double_t;
typedef struct {
  __attribute__((__aligned__(8))) long long __ll;
  long double __ld;
} max_align_t;

#if L'\0' - 1 > 0
typedef unsigned long wchar_t;
#define WCHAR_MAX (0xffffffffu + L'\0')
#define WCHAR_MIN (0 + L'\0')
#else
typedef long wchar_t;
#define WCHAR_MAX (0x7fffffff + L'\0')
#define WCHAR_MIN (-1 - 0x7fffffff + L'\0')
#endif

typedef signed char int8_t;
typedef unsigned char uint8_t;
typedef signed short int16_t;
typedef unsigned short uint16_t;
typedef signed int int32_t;
typedef unsigned int uint32_t;
typedef signed long long int64_t;
typedef unsigned long long uint64_t;
typedef int64_t intmax_t;
typedef uint64_t uintmax_t;

typedef int32_t int_fast8_t;
typedef uint32_t uint_fast8_t;
typedef int32_t int_fast16_t;
typedef uint32_t uint_fast16_t;
typedef int32_t int_fast32_t;
typedef uint32_t uint_fast32_t;
typedef int64_t int_fast64_t;
typedef uint64_t uint_fast64_t;

typedef int8_t int_least8_t;
typedef int16_t int_least16_t;
typedef int32_t int_least32_t;
typedef int64_t int_least64_t;
typedef uint8_t uint_least8_t;
typedef uint16_t uint_least16_t;
typedef uint32_t uint_least32_t;
typedef uint64_t uint_least64_t;

#define INT8_MIN (-1 - 0x7f)
#define INT16_MIN (-1 - 0x7fff)
#define INT32_MIN (-1 - 0x7fffffff)
#define INT64_MIN (-1 - 0x7fffffffffffffff)

#define INT8_MAX (0x7f)
#define INT16_MAX (0x7fff)
#define INT32_MAX (0x7fffffff)
#define INT64_MAX (0x7fffffffffffffff)

#define UINT8_MAX (0xff)
#define UINT16_MAX (0xffff)
#define UINT32_MAX (0xffffffffu)
#define UINT64_MAX (0xffffffffffffffffu)

#define INT_FAST8_MIN INT32_MIN
#define INT_FAST16_MIN INT32_MIN
#define INT_FAST32_MIN INT32_MIN
#define INT_FAST64_MIN INT64_MIN

#define INT_FAST8_MAX INT32_MAX
#define INT_FAST16_MAX INT32_MAX
#define INT_FAST32_MAX INT32_MAX
#define INT_FAST64_MAX INT64_MAX

#define UINT_FAST8_MAX UINT32_MAX
#define UINT_FAST16_MAX UINT32_MAX
#define UINT_FAST32_MAX UINT32_MAX
#define UINT_FAST64_MAX UINT64_MAX

#define INT_LEAST8_MIN INT8_MIN
#define INT_LEAST16_MIN INT16_MIN
#define INT_LEAST32_MIN INT32_MIN
#define INT_LEAST64_MIN INT64_MIN

#define INT_LEAST8_MAX INT8_MAX
#define INT_LEAST16_MAX INT16_MAX
#define INT_LEAST32_MAX INT32_MAX
#define INT_LEAST64_MAX INT64_MAX

#define WCHAR_MAX (0x7fffffff + L'\0')
#define WCHAR_MIN (-1 - 0x7fffffff + L'\0')

#if __cplusplus >= 201103L
#define NULL nullptr
#elif defined(__cplusplus)
#define NULL 0L
#else
#define NULL ((void *)0)
#endif

#define offsetof(type, member) __builtin_offsetof(type, member)

#ifndef __cplusplus

#define true 1
#define false 0
#define bool _Bool

#if __STDC_VERSION__ < 201112L && defined(__GNUC__)
#define _Alignas(t) __attribute__((__aligned__(t)))
#define _Alignof(t) __alignof__(t)
#endif

#define alignas _Alignas
#define alignof _Alignof

#endif

typedef __builtin_va_list va_list;
#define va_start(v, l) __builtin_va_start(v, l)
#define va_end(v) __builtin_va_end(v)
#define va_arg(v, l) __builtin_va_arg(v, l)
#define va_copy(d, s) __builtin_va_copy(d, s)

#ifdef __cplusplus
extern "C" {
#endif

void *memcpy(void *restrict, const void *restrict, size_t);
void *memset(void *, int, size_t);
int memcmp(const void *, const void *, size_t);

char *strcpy(char *restrict, const char *restrict);
char *strncpy(char *restrict, const char *restrict, size_t);
char *strcat(char *restrict, const char *restrict);
char *strncat(char *restrict, const char *restrict, size_t);
int strcmp(const char *, const char *);
int strncmp(const char *, const char *, size_t);
char *strchr(const char *, int);
char *strrchr(const char *, int);
size_t strcspn(const char *, const char *);
size_t strspn(const char *, const char *);
size_t strlen(const char *);
char *strtok(char *restrict, const char *restrict);
int strcasecmp(const char *, const char *);
int strncasecmp(const char *, const char *, size_t);

int isalnum(int);
int isalpha(int);
int isblank(int);
int iscntrl(int);
int isdigit(int);
int isgraph(int);
int islower(int);
int isprint(int);
int ispunct(int);
int isspace(int);
int isupper(int);
int isxdigit(int);
int tolower(int);
int toupper(int);

#ifdef __cplusplus
}
#endif

#endif