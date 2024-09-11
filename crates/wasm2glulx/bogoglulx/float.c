/* float.c: Glulxe code for floating-point operations
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glulxe.h"
#include <math.h>


int init_float() 
{
    /* Check and make sure the native float format is really
       IEEE-754 single-precision. */

    if (sizeof(gfloat32) != 4) {
        fatal_error("gfloat32 is not 32 bits.");
        return FALSE;
    }
    if (encode_float((gfloat32)(-1)) != 0xBF800000) {
        fatal_error("The gfloat32 format of -1 did not match.");
        return FALSE;
    }
    return TRUE;
}

/* Encode and decode floats by reinterpret-casting. */

glui32 encode_float(gfloat32 val)
{
    glui32 res;
    *(gfloat32 *)(&res) = val;
    return res;
}

gfloat32 decode_float(glui32 val)
{
    gfloat32 res;
    *(glui32 *)(&res) = val;
    return res;
}

/* We don't try to implement a reinterpret-cast version of these functions.
   Just seems too risky, what with endianness and who knows what else. */

void encode_double(gfloat64 val, glui32 *reshi, glui32 *reslo)
{
    gfloat64 absval;
    glui32 sign;
    int expo;
    gfloat64 mant;
    glui32 fhi, flo;
 
    if (signbit(val)) {
        sign = 0x80000000;
        absval = -val;
    }
    else {
        sign = 0x0;
        absval = val;
    }

    if (isinf(val)) {
        goto Infinity;
    }

    if (isnan(val)) {
        goto NotANumber;
    }

    mant = frexp(absval, &expo);

    /* Normalize mantissa to be in the range [1.0, 2.0) */
    if (0.5 <= mant && mant < 1.0) {
        mant *= 2.0;
        expo--;
    }
    else if (mant == 0.0) {
        expo = 0;
    }
    else {
        goto Infinity;
    }

    if (expo >= 1024) {
        goto Infinity;
    }
    else if (expo < -1022) {
        /* Denormalized (very small) number */
        mant = ldexp(mant, 1022 + expo);
        expo = 0;
    }
    else if (!(expo == 0 && mant == 0.0)) {
        expo += 1023;
        mant -= 1.0; /* Get rid of leading 1 */
    }

    /* fhi receives the high 28 bits; flo the low 24 bits (total 52 bits) */
    mant *= 268435456.0;          /* 2^28 */
    fhi = (glui32)mant;           /* Truncate */
    mant -= (double)fhi;
    mant *= 16777216.0;           /* 2^24 */
    flo = (glui32)(mant+0.5);     /* Round */
    
    if (flo >> 24) {
        /* The carry propagated out of a string of 24 1 bits. */
        flo = 0;
        fhi++;
        if (fhi >> 28) {
            /* And it also propagated out of the next 28 bits. */
            fhi = 0;
            expo++;
            if (expo >= 255) {
                goto Infinity;
            }
        }
    }

    *reshi = (sign) | ((glui32)(expo << 20)) | ((glui32)(fhi >> 8));
    *reslo = (glui32)((fhi & 0xFF) << 24) | (glui32)(flo);
    return;

 Infinity:
    *reshi = sign | 0x7FF00000;
    *reslo = 0x00000000;
    return;

 NotANumber:
    *reshi = sign | 0x7FF80000;
    *reslo = 0x00000001;
    return;
}

gfloat64 decode_double(glui32 valhi, glui32 vallo)
{
    int sign;
    int expo;
    glui32 manthi, mantlo;
    gfloat64 res;

    /* First byte */
    sign = ((valhi & 0x80000000) != 0);
    expo = (valhi >> 20) & 0x7FF;
    manthi = valhi & 0xFFFFF;
    mantlo = vallo;

    if (expo == 2047) {
        if (manthi == 0 && mantlo == 0) {
            /* Infinity */
            return (sign ? (-INFINITY) : (INFINITY));
        }
        else {
            /* Not a number */
            return (sign ? (-NAN) : (NAN));
        }
    }

    res = (gfloat64)mantlo / 4503599627370496.0 + (gfloat64)manthi / 1048576.0;

    if (expo == 0) {
        expo = -1022;
    }
    else {
        res += 1.0;
        expo -= 1023;
    }
    res = ldexp(res, expo);

    return (sign ? (-res) : (res));
}

