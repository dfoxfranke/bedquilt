/* files.c: Glulxe file-handling code.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glulxe.h"
#include <stdio.h>

/* is_gamefile_valid():
   Check guess what.
*/
int is_gamefile_valid()
{
  unsigned char buf[8];
  int res;
  glui32 version;

  rewind(gamefile);
  res = fread(buf, 1, 8, gamefile);
  
  if (res != 8) {
    fatal_error("This is too short to be a valid Glulx file.");
    return FALSE;
  }

  if (buf[0] != 'G' || buf[1] != 'l' || buf[2] != 'u' || buf[3] != 'l') {
    fatal_error("This is not a valid Glulx file.");
    return FALSE;
  }

  /* We support version 2.0 through 3.1.*. */

  version = Read4(buf+4);
  if (version < 0x20000) {
    fatal_error("This Glulx file is too old a version to execute.");
    return FALSE;
  }
  if (version >= 0x30200) {
    fatal_error("This Glulx file is too new a version to execute.");
    return FALSE;
  }

  return TRUE;
}
