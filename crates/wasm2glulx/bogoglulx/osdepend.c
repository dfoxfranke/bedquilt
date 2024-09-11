/* osdepend.c: Glulxe platform-dependent code.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glulxe.h"
#include <stdlib.h>
#include <math.h>

/* Allocate a chunk of memory. */
void *glulx_malloc(glui32 len)
{
  return malloc(len);
}

/* Resize a chunk of memory. This must follow ANSI rules: if the
   size-change fails, this must return NULL, but the original chunk
   must remain unchanged. */
void *glulx_realloc(void *ptr, glui32 len)
{
  return realloc(ptr, len);
}

/* Deallocate a chunk of memory. */
void glulx_free(void *ptr)
{
  free(ptr);
}

/* I'm putting a wrapper for qsort() here, in case I ever have to
   worry about a platform without it. But I am not worrying at
   present. */
void glulx_sort(void *addr, int count, int size, 
  int (*comparefunc)(void *p1, void *p2))
{
  qsort(addr, count, size, (int (*)(const void *, const void *))comparefunc);
}

/* This wrapper handles all special cases, even if the underlying
   powf() function doesn't. */
gfloat32 glulx_powf(gfloat32 val1, gfloat32 val2)
{
  if (val1 == 1.0f)
    return 1.0f;
  else if ((val2 == 0.0f) || (val2 == -0.0f))
    return 1.0f;
  else if ((val1 == -1.0f) && isinf(val2))
    return 1.0f;
  return powf(val1, val2);
}

/* Same for pow(). */
extern gfloat64 glulx_pow(gfloat64 val1, gfloat64 val2)
{
  if (val1 == 1.0)
    return 1.0;
  else if ((val2 == 0.0) || (val2 == -0.0))
    return 1.0;
  else if ((val1 == -1.0) && isinf(val2))
    return 1.0;
  return pow(val1, val2);
}