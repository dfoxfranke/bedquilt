/* Implementations of just enough of the C standard library to be able to
compile us. Extracted from MUSL.

SPDX-FileCopyrightText: Copyright © 2005-2020 Rich Felker, et al., Copyright © 2024 Daniel Fox Franke
SPDX-License-Identifier: MIT
*/

#include "bedquilt.h"

int isalnum(int c) { return isalpha(c) || isdigit(c); }
int isalpha(int c) { return ((unsigned)c | 32) - 'a' < 26; }
int isascii(int c) { return !(c & ~0x7f); }
int isblank(int c) { return (c == ' ' || c == '\t'); }
int iscntrl(int c) { return (unsigned)c < 0x20 || c == 0x7f; }
int isdigit(int c) { return (unsigned)c - '0' < 10; }
int isgraph(int c) { return (unsigned)c - 0x21 < 0x5e; }
int islower(int c) { return (unsigned)c - 'a' < 26; }
int isprint(int c) { return (unsigned)c - 0x20 < 0x5f; }
int ispunct(int c) { return isgraph(c) && !isalnum(c); }
int isspace(int c) { return c == ' ' || (unsigned)c - '\t' < 5; }
int isupper(int c) { return (unsigned)c - 'A' < 26; }

int tolower(int c) {
  if (isupper(c))
    return c | 32;
  return c;
}

int toupper(int c) {
  if (islower(c))
    return c & 0x5f;
  return c;
}

void *memcpy(void *restrict dest, const void *restrict src, size_t n) {
  unsigned char *d = dest;
  const unsigned char *s = src;
  for (; n; n--)
    *d++ = *s++;
  return dest;
}

void *memset(void *dest, int c, size_t n) {
  unsigned char *s = dest;
  for (; n; n--, s++)
    *s = c;
  return dest;
}

int memcmp(const void *vl, const void *vr, size_t n) {
  const unsigned char *l = vl, *r = vr;
  for (; n && *l == *r; n--, l++, r++)
    ;
  return n ? *l - *r : 0;
}

char *strcpy(char *restrict d, const char *restrict s) {
  for (; (*d = *s); s++, d++)
    ;
  return d;
}

char *strncpy(char *restrict d, const char *restrict s, size_t n) {
  for (; n && (*d = *s); n--, s++, d++)
    ;
  memset(d, 0, n);
  return d;
}

char *strcat(char *restrict dest, const char *restrict src) {
  strcpy(dest + strlen(dest), src);
  return dest;
}

char *strncat(char *restrict d, const char *restrict s, size_t n) {
  char *a = d;
  d += strlen(d);
  while (n && *s)
    n--, *d++ = *s++;
  *d++ = 0;
  return a;
}

int strcmp(const char *l, const char *r) {
  for (; *l == *r && *l; l++, r++)
    ;
  return *(unsigned char *)l - *(unsigned char *)r;
}

int strncmp(const char *_l, const char *_r, size_t n) {
  const unsigned char *l = (void *)_l, *r = (void *)_r;
  if (!n--)
    return 0;
  for (; *l && *r && n && *l == *r; l++, r++, n--)
    ;
  return *l - *r;
}

static char *strchrnul(const char *s, int c) {
  c = (unsigned char)c;
  if (!c)
    return (char *)s + strlen(s);
  for (; *s && *(unsigned char *)s != c; s++)
    ;
  return (char *)s;
}

char *strchr(const char *s, int c) {
  char *r = strchrnul(s, c);
  return *(unsigned char *)r == (unsigned char)c ? r : 0;
}

char *strrchr(const char *s, int c) {
  size_t n = strlen(s) + 1;
  while (n--)
    if (s[n] == c)
      return (char *)(s + n);
  return NULL;
}

size_t strlen(const char *s) {
  const char *a = s;
  for (; *s; s++)
    ;
  return s - a;
}

#define BITOP(a, b, op)                                                        \
  ((a)[(size_t)(b) / (8 * sizeof *(a))] op(size_t) 1                           \
   << ((size_t)(b) % (8 * sizeof *(a))))

size_t strspn(const char *s, const char *c) {
  const char *a = s;
  size_t byteset[32 / sizeof(size_t)] = {0};

  if (!c[0])
    return 0;
  if (!c[1]) {
    for (; *s == *c; s++)
      ;
    return s - a;
  }

  for (; *c && BITOP(byteset, *(unsigned char *)c, |=); c++)
    ;
  for (; *s && BITOP(byteset, *(unsigned char *)s, &); s++)
    ;
  return s - a;
}

size_t strcspn(const char *s, const char *c) {
  const char *a = s;
  size_t byteset[32 / sizeof(size_t)];

  if (!c[0] || !c[1])
    return strchrnul(s, *c) - a;

  memset(byteset, 0, sizeof byteset);
  for (; *c && BITOP(byteset, *(unsigned char *)c, |=); c++)
    ;
  for (; *s && !BITOP(byteset, *(unsigned char *)s, &); s++)
    ;
  return s - a;
}

char *strtok(char *restrict s, const char *restrict sep) {
  static char *p;
  if (!s && !(s = p))
    return NULL;
  s += strspn(s, sep);
  if (!*s)
    return p = 0;
  p = s + strcspn(s, sep);
  if (*p)
    *p++ = 0;
  else
    p = 0;
  return s;
}

int strcasecmp(const char *_l, const char *_r) {
  const unsigned char *l = (void *)_l, *r = (void *)_r;
  for (; *l && *r && (*l == *r || tolower(*l) == tolower(*r)); l++, r++)
    ;
  return tolower(*l) - tolower(*r);
}

int strncasecmp(const char *_l, const char *_r, size_t n) {
  const unsigned char *l = (void *)_l, *r = (void *)_r;
  if (!n--)
    return 0;
  for (; *l && *r && n && (*l == *r || tolower(*l) == tolower(*r));
       l++, r++, n--)
    ;
  return tolower(*l) - tolower(*r);
}