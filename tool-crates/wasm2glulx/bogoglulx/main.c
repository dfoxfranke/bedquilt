/* main.c: Glulxe top-level code.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glulxe.h"
#include <stdio.h>
#include <stdlib.h>

#define TRAP_LEN 11
static const char* trap_messages[TRAP_LEN] = {
  "unreachable",
  "integer overflow",
  "integer divide by zero",
  "invalid conversion to integer",
  "out of bounds memory access",
  "indirect call type mismatch",
  "out of bounds table access",
  "undefined element",
  "uninitialized element",
  "call stack exhausted",
  "unknown trap code",
};

FILE *gamefile = NULL; /* The stream containing the Glulx file. */

int main(int argc, char *argv[])
{
  if (argc != 2) {
    fprintf(stderr, "Usage: bogoglulx gamefile.ulx\n");
    return EXIT_FAILURE;
  }

  gamefile = fopen(argv[1], "rb");
  if (gamefile == NULL) {
    perror("fopen");
    return EXIT_FAILURE;
  }


  if (!is_gamefile_valid()) {
    /* The fatal error has already been displayed. */
    return EXIT_FAILURE;
  }

  if (!init_float()) {
    return EXIT_FAILURE;
  }
  
  setup_vm();
  execute_loop();
  finalize_vm();

  fclose(gamefile);
  return EXIT_SUCCESS;
}

/* fatal_error_handler():
   Display an error in the error window, and then exit.
*/
void fatal_error_handler(char *str, int useval, glsi32 val)
{
  if (useval) {
    printf("?%s: %x", str, (unsigned int)val);
  } else {
    printf("?%s", str);
  }
  exit(EXIT_FAILURE);
}

void trap(int code) {
  if (code >= TRAP_LEN || code < 0) {
    code = TRAP_LEN - 1;
  }

  printf("!%s", trap_messages[code]);
  exit(EXIT_FAILURE);
}
