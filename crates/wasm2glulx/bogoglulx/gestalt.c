/* gestalt.c: Glulxe code for gestalt selectors
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glulxe.h"
#include "gestalt.h"

glui32 do_gestalt(glui32 val, glui32 val2)
{
  switch (val) {

  case gestulx_GlulxVersion:
    return 0x00030103; /* Glulx spec version 3.1.3 */

  case gestulx_TerpVersion:
    return 0x00000601; /* Glulxe version 0.6.1 */

  case gestulx_ResizeMem:
    return 1; /* We can handle setmemsize. */

  case gestulx_MemCopy:
    return 1; /* We can do mcopy/mzero. */

  case gestulx_MAlloc:
    return 1; /* We can handle malloc/mfree. */

  case gestulx_MAllocHeap:
    return heap_get_start();

  case gestulx_Acceleration:
    return 1; /* We can do accelfunc/accelparam. */

  case gestulx_Float:
    return 1; /* We can do floating-point operations. */

  case gestulx_Double:
    return 1; /* We can do double-precision operations. */

  default:
    return 0;

  }
}
