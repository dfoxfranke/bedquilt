/*
 * Saving and resuming.
 *
 * (ESR) This replaces  a bunch of particularly nasty FORTRAN-derived code;
 * see the history.adoc file in the source distribution for discussion.
 *
 * SPDX-FileCopyrightText: (C) 1977, 2005 by Will Crowther and Don Woods
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include "advent.h"
#include "bedquilt.h"
#include "wasmglk.h"

/*
 * Use this to detect endianness mismatch.  Can't be unchanged by byte-swapping.
 */
#define ENDIAN_MAGIC 2317

struct save_t save;

static void savefile(strid_t str) {
  /* Save game to file. No input or output from user. */
  memcpy(&save.magic, ADVENT_MAGIC, sizeof(ADVENT_MAGIC));
  if (save.version == 0) {
    save.version = SAVE_VERSION;
  }
  if (save.canary == 0) {
    save.canary = ENDIAN_MAGIC;
  }
  save.game = game;
  glk_put_buffer_stream(str, (char *)&save, sizeof(struct save_t));
}

static int restore(strid_t str) {
  /*  Read and restore game state from file, assuming
   *  sane initial state.
   */

  glui32 read_size =
      glk_get_buffer_stream(str, (char *)&save, sizeof(struct save_t));
  glk_stream_close(str, NULL);

  if ((size_t)read_size != sizeof(struct save_t)) {
    rspeak(BAD_SAVE);
  } else if (memcmp(save.magic, ADVENT_MAGIC, sizeof(ADVENT_MAGIC)) != 0 ||
             save.canary != ENDIAN_MAGIC) {
    rspeak(BAD_SAVE);
  } else if (save.version != SAVE_VERSION) {
    rspeak(VERSION_SKEW, save.version / 10, MOD(save.version, 10),
           SAVE_VERSION / 10, MOD(SAVE_VERSION, 10));
  } else if (!is_valid(save.game)) {
    rspeak(SAVE_TAMPERING);
    glk_exit();
  } else {
    game = save.game;
  }
  return GO_TOP;
}

/* Suspend and resume */

int suspend(void) {
  /*  Suspend.  Offer to save things in a file, but charging
   *  some points (so can't win by using saved games to retry
   *  battles or to start over after learning zzword).
   *  If ADVENT_NOSAVE is defined, gripe instead. */

  frefid_t fref;
  strid_t str;

  rspeak(SUSPEND_WARNING);
  if (!yes_or_no(arbitrary_messages[THIS_ACCEPTABLE],
                 arbitrary_messages[OK_MAN], arbitrary_messages[OK_MAN])) {
    return GO_CLEAROBJ;
  }

  do {
    fref = glk_fileref_create_by_prompt(fileusage_SavedGame, filemode_Write, 0);
    if (fref == NULL) {
      glk_put_string("Suspension cancelled.\n");
      return GO_CLEAROBJ;
    }

    str = glk_stream_open_file(fref, filemode_Write, 0);
    if (str == NULL) {
      glk_put_string("Can't open save file, try again.\n");
    }
  } while (str == NULL);

  game.saved = game.saved + 5;
  savefile(str);

  glk_stream_close(str, NULL);
  glk_fileref_destroy(fref);
  rspeak(RESUME_HELP);
  glk_exit();
}

int resume(void) {
  /*  Resume.  Read a suspended game back from a file.
   *  If ADVENT_NOSAVE is defined, gripe instead. */

  frefid_t fref;
  strid_t str;

  if (game.loc != LOC_START || game.locs[LOC_START].abbrev != 1) {
    rspeak(RESUME_ABANDON);
    if (!yes_or_no(arbitrary_messages[THIS_ACCEPTABLE],
                   arbitrary_messages[OK_MAN], arbitrary_messages[OK_MAN])) {
      return GO_CLEAROBJ;
    }
  }

  do {
    fref = glk_fileref_create_by_prompt(fileusage_SavedGame, filemode_Read, 0);
    if (fref == NULL) {
      glk_put_string("Resumption cancelled.\n");
      return GO_CLEAROBJ;
    }

    str = glk_stream_open_file(fref, filemode_Read, 0);
    if (str == NULL) {
      glk_put_string("Can't open save file, try again.\n");
    }
  } while (str == NULL);
  glk_fileref_destroy(fref);

  return restore(str);
}

bool is_valid(struct game_t valgame) {
  /*  Save files can be roughly grouped into three groups:
   *  With valid, reachable state, with valid, but unreachable
   *  state and with invalid state. We check that state is
   *  valid: no states are outside minimal or maximal value
   */

  /* Prevent division by zero */
  if (valgame.abbnum == 0) {
    return false; // LCOV_EXCL_LINE
  }

  /*  Bounds check for locations */
  if (valgame.chloc < -1 || valgame.chloc > NLOCATIONS || valgame.chloc2 < -1 ||
      valgame.chloc2 > NLOCATIONS || valgame.loc < 0 ||
      valgame.loc > NLOCATIONS || valgame.newloc < 0 ||
      valgame.newloc > NLOCATIONS || valgame.oldloc < 0 ||
      valgame.oldloc > NLOCATIONS || valgame.oldlc2 < 0 ||
      valgame.oldlc2 > NLOCATIONS) {
    return false; // LCOV_EXCL_LINE
  }
  /*  Bounds check for location arrays */
  for (int i = 0; i <= NDWARVES; i++) {
    if (valgame.dwarves[i].loc < -1 || valgame.dwarves[i].loc > NLOCATIONS ||
        valgame.dwarves[i].oldloc < -1 ||
        valgame.dwarves[i].oldloc > NLOCATIONS) {
      return false; // LCOV_EXCL_LINE
    }
  }

  for (int i = 0; i <= NOBJECTS; i++) {
    if (valgame.objects[i].place < -1 ||
        valgame.objects[i].place > NLOCATIONS ||
        valgame.objects[i].fixed < -1 ||
        valgame.objects[i].fixed > NLOCATIONS) {
      return false; // LCOV_EXCL_LINE
    }
  }

  /*  Bounds check for dwarves */
  if (valgame.dtotal < 0 || valgame.dtotal > NDWARVES || valgame.dkill < 0 ||
      valgame.dkill > NDWARVES) {
    return false; // LCOV_EXCL_LINE
  }

  /*  Validate that we didn't die too many times in save */
  if (valgame.numdie >= NDEATHS) {
    return false; // LCOV_EXCL_LINE
  }

  /* Recalculate tally, throw the towel if in disagreement */
  int temp_tally = 0;
  for (int treasure = 1; treasure <= NOBJECTS; treasure++) {
    if (objects[treasure].is_treasure) {
      if (OBJECT_IS_NOTFOUND2(valgame, treasure)) {
        ++temp_tally;
      }
    }
  }
  if (temp_tally != valgame.tally) {
    return false; // LCOV_EXCL_LINE
  }

  /* Check that properties of objects aren't beyond expected */
  for (obj_t obj = 0; obj <= NOBJECTS; obj++) {
    if (PROP_IS_INVALID(valgame.objects[obj].prop)) {
      return false; // LCOV_EXCL_LINE
    }
  }

  /* Check that values in linked lists for objects in locations are inside
   * bounds */
  for (loc_t loc = LOC_NOWHERE; loc <= NLOCATIONS; loc++) {
    if (valgame.locs[loc].atloc < NO_OBJECT ||
        valgame.locs[loc].atloc > NOBJECTS * 2) {
      return false; // LCOV_EXCL_LINE
    }
  }
  for (obj_t obj = 0; obj <= NOBJECTS * 2; obj++) {
    if (valgame.link[obj] < NO_OBJECT || valgame.link[obj] > NOBJECTS * 2) {
      return false; // LCOV_EXCL_LINE
    }
  }

  return true;
}

/* end */
