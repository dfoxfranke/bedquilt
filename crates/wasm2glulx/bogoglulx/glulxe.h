/* glulxe.h: Glulxe header file.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#ifndef _GLULXE_H
#define _GLULXE_H

/* We define our own TRUE and FALSE and NULL, because ANSI
   is a strange world. */
#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif
#ifndef NULL
#define NULL 0
#endif

#include <stdio.h>
#include <stdint.h>
typedef uint32_t glui32;
typedef int32_t glsi32;
typedef uint16_t glui16;
typedef int16_t glsi16;

/* Some macros to read and write integers to memory, always in big-endian
   format. */
#define Read4(ptr)    \
  ( (glui32)((glui32)((unsigned char *)(ptr))[0] << 24)  \
  | (glui32)((glui32)((unsigned char *)(ptr))[1] << 16)  \
  | (glui32)((glui32)((unsigned char *)(ptr))[2] << 8)   \
  | (glui32)((glui32)((unsigned char *)(ptr))[3]))
#define Read2(ptr)    \
  ( (glui16)(((unsigned char *)(ptr))[0] << 8)  \
  | (glui16)(((unsigned char *)(ptr))[1]))
#define Read1(ptr)    \
  ((unsigned char)(((unsigned char *)(ptr))[0]))

#define Write4(ptr, vl)   \
  (((ptr)[0] = (unsigned char)(((glui32)(vl)) >> 24)),   \
   ((ptr)[1] = (unsigned char)(((glui32)(vl)) >> 16)),   \
   ((ptr)[2] = (unsigned char)(((glui32)(vl)) >> 8)),    \
   ((ptr)[3] = (unsigned char)(((glui32)(vl)))))
#define Write2(ptr, vl)   \
  (((ptr)[0] = (unsigned char)(((glui32)(vl)) >> 8)),   \
   ((ptr)[1] = (unsigned char)(((glui32)(vl)))))
#define Write1(ptr, vl)   \
  (((unsigned char *)(ptr))[0] = (vl))

#define Verify(adr, ln) verify_address(adr, ln)
#define VerifyW(adr, ln) verify_address_write(adr, ln)
#define VerifyStk(adr, ln) verify_address_stack(adr, ln)

#define Mem1(adr)  (Verify(adr, 1), Read1(memmap+(adr)))
#define Mem2(adr)  (Verify(adr, 2), Read2(memmap+(adr)))
#define Mem4(adr)  (Verify(adr, 4), Read4(memmap+(adr)))
#define MemW1(adr, vl)  (VerifyW(adr, 1), Write1(memmap+(adr), (vl)))
#define MemW2(adr, vl)  (VerifyW(adr, 2), Write2(memmap+(adr), (vl)))
#define MemW4(adr, vl)  (VerifyW(adr, 4), Write4(memmap+(adr), (vl)))

/* Macros to access values on the stack. These *must* be used 
   with proper alignment! (That is, Stk4 and StkW4 must take 
   addresses which are multiples of four, etc.) If the alignment
   rules are not followed, the program will see performance
   degradation or even crashes, depending on the machine CPU. */

#define Stk1(adr)   \
  (VerifyStk(adr, 1), *((unsigned char *)(stack+(adr))))
#define Stk2(adr)   \
  (VerifyStk(adr, 2), *((glui16 *)(stack+(adr))))
#define Stk4(adr)   \
  (VerifyStk(adr, 4), *((glui32 *)(stack+(adr))))

#define StkW1(adr, vl)   \
  (VerifyStk(adr, 1), *((unsigned char *)(stack+(adr))) = (unsigned char)(vl))
#define StkW2(adr, vl)   \
  (VerifyStk(adr, 2), *((glui16 *)(stack+(adr))) = (glui16)(vl))
#define StkW4(adr, vl)   \
  (VerifyStk(adr, 4), *((glui32 *)(stack+(adr))) = (glui32)(vl))

/* Some useful structures. */

/* oparg_t:
   Represents one operand value to an instruction being executed. The
   code in exec.c assumes that no instruction has more than MAX_OPERANDS
   of these.
*/
typedef struct oparg_struct {
  glui32 desttype;
  glui32 value;
} oparg_t;

#define MAX_OPERANDS (8)

/* operandlist_t:
   Represents the operand structure of an opcode.
*/
typedef struct operandlist_struct {
  int num_ops; /* Number of operands for this opcode */
  int arg_size; /* Usually 4, but can be 1 or 2 */
  int *formlist; /* Array of values, either modeform_Load or modeform_Store */
} operandlist_t;
#define modeform_Load (1)
#define modeform_Store (2)

/* Some useful globals */

extern FILE* gamefile;
extern glui32 gamefile_start, gamefile_len;

extern unsigned char *memmap;
extern unsigned char *stack;

extern glui32 ramstart;
extern glui32 endgamefile;
extern glui32 origendmem;
extern glui32 stacksize;
extern glui32 startfuncaddr;
extern glui32 checksum;
extern glui32 stackptr;
extern glui32 frameptr;
extern glui32 pc;
extern glui32 valstackbase;
extern glui32 localsbase;
extern glui32 endmem;
extern glui32 prevpc;

/* main.c */
extern void fatal_error_handler(char *str, int useval, glsi32 val) __attribute__((noreturn));
#define fatal_error(s)  (fatal_error_handler((s), FALSE, 0))
#define fatal_error_i(s, v)  (fatal_error_handler((s), TRUE, (v)))
extern void trap(int code) __attribute__((noreturn));

/* files.c */
extern int is_gamefile_valid(void);

/* vm.c */
extern void setup_vm(void);
extern void finalize_vm(void);
extern void vm_restart(void);
extern glui32 change_memsize(glui32 newlen, int internal);
extern glui32 *pop_arguments(glui32 count, glui32 addr);
extern void verify_address(glui32 addr, glui32 count);
extern void verify_address_write(glui32 addr, glui32 count);
extern void verify_address_stack(glui32 stackpos, glui32 count);
extern void verify_array_addresses(glui32 addr, glui32 count, glui32 size);

/* exec.c */
extern void execute_loop(void);

/* operand.c */
extern const operandlist_t *fast_operandlist[0x80];
extern void init_operands(void);
extern const operandlist_t *lookup_operandlist(glui32 opcode);
extern void parse_operands(oparg_t *opargs, const operandlist_t *oplist);
extern void store_operand(glui32 desttype, glui32 destaddr, glui32 storeval);
extern void store_operand_s(glui32 desttype, glui32 destaddr, glui32 storeval);
extern void store_operand_b(glui32 desttype, glui32 destaddr, glui32 storeval);

/* funcs.c */
extern void enter_function(glui32 addr, glui32 argc, glui32 *argv);
extern void leave_function(void);
extern void push_callstub(glui32 desttype, glui32 destaddr);
extern void pop_callstub(glui32 returnvalue);
extern glui32 pop_callstub_string(int *bitnum);

/* heap.c */
extern void heap_clear(void);
extern int heap_is_active(void);
extern glui32 heap_get_start(void);
extern glui32 heap_alloc(glui32 len);
extern void heap_free(glui32 addr);
extern int heap_get_summary(glui32 *valcount, glui32 **summary);
extern int heap_apply_summary(glui32 valcount, glui32 *summary);
extern void heap_sanity_check(void);

/* search.c */
extern glui32 linear_search(glui32 key, glui32 keysize, 
  glui32 start, glui32 structsize, glui32 numstructs, 
  glui32 keyoffset, glui32 options);
extern glui32 binary_search(glui32 key, glui32 keysize, 
  glui32 start, glui32 structsize, glui32 numstructs, 
  glui32 keyoffset, glui32 options);
extern glui32 linked_search(glui32 key, glui32 keysize, 
  glui32 start, glui32 keyoffset, glui32 nextoffset,
  glui32 options);

/* osdepend.c */
extern void *glulx_malloc(glui32 len);
extern void *glulx_realloc(void *ptr, glui32 len);
extern void glulx_free(void *ptr);
extern void glulx_sort(void *addr, int count, int size, 
  int (*comparefunc)(void *p1, void *p2));

/* gestalt.c */
extern glui32 do_gestalt(glui32 val, glui32 val2);

/* You may have to edit the definition of gfloat32 to make sure it's really
   a 32-bit floating-point type. */
typedef float gfloat32;

/* float.c */
extern int init_float(void);
extern glui32 encode_float(gfloat32 val);
extern gfloat32 decode_float(glui32 val);

extern gfloat32 glulx_powf(gfloat32 val1, gfloat32 val2);


/* You may have to edit the definition of gfloat64 to make sure it's really
   a 64-bit floating-point type. */
typedef double gfloat64;

extern void encode_double(gfloat64 val, glui32 *reshi, glui32 *reslo);
extern gfloat64 decode_double(glui32 valhi, glui32 vallo);

extern gfloat64 glulx_pow(gfloat64 val1, gfloat64 val2);

#define TRAP_INTEGER_OVERFLOW 1
#define TRAP_INTEGER_DIVIDE_BY_ZERO 2
#define TRAP_STACK_EXHAUSTED 9

#endif /* _GLULXE_H */
