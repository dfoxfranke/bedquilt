/* funcs.c: Glulxe function-handling functions.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glulxe.h"

/* enter_function():
   This writes a new call frame onto the stack, at stackptr. It leaves
   frameptr pointing to the frame (ie, the original stackptr value.) 
   argc and argv are an array of arguments. Note that if argc is zero,
   argv may be NULL.
*/
void enter_function(glui32 funcaddr, glui32 argc, glui32 *argv)
{
  int ix, jx;
  int locallen;
  int functype;
  glui32 modeaddr, opaddr, val;
  int loctype, locnum;
  glui32 addr = funcaddr;

  /* Check the Glulx type identifier byte. */
  functype = Mem1(addr);
  if (functype != 0xC0 && functype != 0xC1) {
    if (functype >= 0xC0 && functype <= 0xDF)
      fatal_error_i("Call to unknown type of function.", addr);
    else
      fatal_error_i("Call to non-function.", addr);
  }
  addr++;

  /* Bump the frameptr to the top. */
  frameptr = stackptr;

  /* Go through the function's locals-format list, copying it to the
     call frame. At the same time, we work out how much space the locals
     will actually take up. (Including padding.) */
  ix = 0;
  locallen = 0;
  while (1) {
    /* Grab two bytes from the locals-format list. These are 
       unsigned (0..255 range). */
    loctype = Mem1(addr);
    addr++;
    locnum = Mem1(addr);
    addr++;

    /* Copy them into the call frame. */
    StkW1(frameptr+8+2*ix, loctype);
    StkW1(frameptr+8+2*ix+1, locnum);
    ix++;

    /* If the type is zero, we're done, except possibly for two more
       zero bytes in the call frame (to ensure 4-byte alignment.) */
    if (loctype == 0) {
      /* Make sure ix is even. */
      if (ix & 1) {
        StkW1(frameptr+8+2*ix, 0);
        StkW1(frameptr+8+2*ix+1, 0);
        ix++;
      }
      break;
    }

    /* Pad to 4-byte or 2-byte alignment if these locals are 4 or 2
       bytes long. */
    if (loctype == 4) {
      while (locallen & 3)
        locallen++;
    }
    else if (loctype == 2) {
      while (locallen & 1)
        locallen++;
    }
    else if (loctype == 1) {
      /* no padding */
    }
    else {
      fatal_error("Illegal local type in locals-format list.");
    }

    /* Add the length of the locals themselves. */
    locallen += (loctype * locnum);
  }

  /* Pad the locals to 4-byte alignment. */
  while (locallen & 3)
    locallen++;

  /* We now know how long the locals-frame and locals segments are. */
  localsbase = frameptr+8+2*ix;
  valstackbase = localsbase+locallen;

  /* Test for stack overflow. */
  /* This really isn't good enough; if the format list overflowed the
     stack, we've already written outside the stack array. */
  if (valstackbase >= stacksize)
    trap(TRAP_STACK_EXHAUSTED);

  /* Fill in the beginning of the stack frame. */
  StkW4(frameptr+4, 8+2*ix);
  StkW4(frameptr, 8+2*ix+locallen);

  /* Set the stackptr and PC. */
  stackptr = valstackbase;
  pc = addr;

  /* Zero out all the locals. */
  for (jx=0; jx<locallen; jx++) 
    StkW1(localsbase+jx, 0);

  if (functype == 0xC0) {
    /* Push the function arguments on the stack. The locals have already
       been zeroed. */
    if (stackptr+4*(argc+1) >= stacksize)
      trap(TRAP_STACK_EXHAUSTED);
    for (ix=0; ix<argc; ix++) {
      val = argv[(argc-1)-ix];
      StkW4(stackptr, val);
      stackptr += 4;
    }
    StkW4(stackptr, argc);
    stackptr += 4;
  }
  else {
    /* Copy in function arguments. This is a bit gross, since we have to
       follow the locals format. If there are fewer arguments than locals,
       that's fine -- we've already zeroed out this space. If there are
       more arguments than locals, the extras are silently dropped. */
    modeaddr = frameptr+8;
    opaddr = localsbase;
    ix = 0;
    while (ix < argc) {
      loctype = Stk1(modeaddr);
      modeaddr++;
      locnum = Stk1(modeaddr);
      modeaddr++;
      if (loctype == 0)
        break;
      if (loctype == 4) {
        while (opaddr & 3)
          opaddr++;
        while (ix < argc && locnum) {
          val = argv[ix];
          StkW4(opaddr, val);
          opaddr += 4;
          ix++;
          locnum--;
        }
      }
      else if (loctype == 2) {
        while (opaddr & 1)
          opaddr++;
        while (ix < argc && locnum) {
          val = argv[ix] & 0xFFFF;
          StkW2(opaddr, val);
          opaddr += 2;
          ix++;
          locnum--;
        }
      }
      else if (loctype == 1) {
        while (ix < argc && locnum) {
          val = argv[ix] & 0xFF;
          StkW1(opaddr, val);
          opaddr += 1;
          ix++;
          locnum--;
        }
      }
    }
  }
}

/* leave_function():
   Pop the current call frame off the stack. This is very simple.
*/
void leave_function()
{
  stackptr = frameptr;
}

/* push_callstub():
   Push the magic four values on the stack: result destination,
   PC, and frameptr. 
*/
void push_callstub(glui32 desttype, glui32 destaddr)
{
  if (stackptr+16 > stacksize)
    trap(TRAP_STACK_EXHAUSTED);
  StkW4(stackptr+0, desttype);
  StkW4(stackptr+4, destaddr);
  StkW4(stackptr+8, pc);
  StkW4(stackptr+12, frameptr);
  stackptr += 16;
}

/* pop_callstub():
   Remove the magic four values from the stack, and use them. The
   returnvalue, whatever it is, is put at the result destination;
   the PC and frameptr registers are set.
*/
void pop_callstub(glui32 returnvalue)
{
  glui32 desttype, destaddr;
  glui32 newpc, newframeptr;

  if (stackptr < 16)
    fatal_error("Stack underflow in callstub.");
  stackptr -= 16;

  newframeptr = Stk4(stackptr+12);
  newpc = Stk4(stackptr+8);
  destaddr = Stk4(stackptr+4);
  desttype = Stk4(stackptr+0);

  pc = newpc;
  frameptr = newframeptr;

  /* Recompute valstackbase and localsbase */
  valstackbase = frameptr + Stk4(frameptr);
  localsbase = frameptr + Stk4(frameptr+4);

  switch (desttype) {

  case 0x11:
  case 0x10:
  case 0x12:
  case 0x13:
  case 0x14:
    fatal_error("Unexpected callstub type.");
    break;

  default:
    /* We're back in the original frame, so we can store the returnvalue. 
       (If we tried to do this before resetting frameptr, a result
       destination on the stack would go astray.) */
    store_operand(desttype, destaddr, returnvalue);
    break;
  }
}

/* pop_callstub_string():
   Remove the magic four values, but interpret them as a string restart
   state. Returns zero if it's a termination stub, or returns the
   restart address. The bitnum is extra.
*/
glui32 pop_callstub_string(int *bitnum)
{
  glui32 desttype, destaddr, newpc;

  if (stackptr < 16)
    fatal_error("Stack underflow in callstub.");
  stackptr -= 16;

  newpc = Stk4(stackptr+8);
  destaddr = Stk4(stackptr+4);
  desttype = Stk4(stackptr+0);

  pc = newpc;

  if (desttype == 0x11) {
    return 0;
  }
  if (desttype == 0x10) {
    *bitnum = destaddr;
    return pc;
  }

  fatal_error("Function-terminator call stub at end of string.");
  return 0;
}

