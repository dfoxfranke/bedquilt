/*
 * Initialisation
 *
 * SPDX-FileCopyrightText: (C) 1977, 2005 by Will Crowther and Don Woodsm
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include "advent.h"
#include "bedquilt.h"
#include "wasmglk.h"

struct game_t game = {
    /*  Last dwarf is special (the pirate).  He always starts at his
     *  chest's eventual location inside the maze. This loc is saved
     *  in chloc for ref. The dead end in the other maze has its
     *  loc stored in chloc2. */
    .chloc = LOC_MAZEEND12, .chloc2 = LOC_DEADEND13, .abbnum = 5,
    .clock1 = WARNTIME,     .clock2 = FLASHTIME,     .newloc = LOC_START,
    .loc = LOC_START,       .limit = GAMELIMIT,      .foobar = WORD_EMPTY,
};

void initialise(void) {
  glk_put_string("Initialising...\n");

  game.lcg_x = 0;
  for (int i = 0; i < 5; ++i) {
    game.zzword[i] = 'A' + glulx_random(26);
  }
  game.zzword[1] = '\''; // force second char to apostrophe
  game.zzword[5] = '\0';

  for (int i = 1; i <= NDWARVES; i++) {
    game.dwarves[i].loc = dwarflocs[i - 1];
  }

  for (int i = 1; i <= NOBJECTS; i++) {
    game.objects[i].place = LOC_NOWHERE;
  }

  for (int i = 1; i <= NLOCATIONS; i++) {
    if (!(locations[i].description.big == 0 || tkey[i] == 0)) {
      int k = tkey[i];
      if (travel[k].motion == HERE) {
        conditions[i] |= (1 << COND_FORCED);
      }
    }
  }

  /*  Set up the game.locs atloc and game.link arrays.
   *  We'll use the DROP subroutine, which prefaces new objects on the
   *  lists.  Since we want things in the other order, we'll run the
   *  loop backwards.  If the object is in two locs, we drop it twice.
   *  Also, since two-placed objects are typically best described
   *  last, we'll drop them first. */
  for (int i = NOBJECTS; i >= 1; i--) {
    if (objects[i].fixd > 0) {
      drop(i + NOBJECTS, objects[i].fixd);
      drop(i, objects[i].plac);
    }
  }

  for (int i = 1; i <= NOBJECTS; i++) {
    int k = NOBJECTS + 1 - i;
    game.objects[k].fixed = objects[k].fixd;
    if (objects[k].plac != 0 && objects[k].fixd <= 0) {
      drop(k, objects[k].plac);
    }
  }

  /*  Treasure props are initially STATE_NOTFOUND, and are set to
   *  STATE_FOUND the first time they are described.  game.tally
   *  keeps track of how many are not yet found, so we know when to
   *  close the cave.
   *  (ESR) Non-trreasures are set to STATE_FOUND explicity so we
   *  don't rely on the value of uninitialized storage. This is to
   *  make translation to future languages easier. */
  for (int object = 1; object <= NOBJECTS; object++) {
    if (objects[object].is_treasure) {
      ++game.tally;
      if (objects[object].inventory != NULL) {
        OBJECT_SET_NOT_FOUND(object);
      }
    } else {
      OBJECT_SET_FOUND(object);
    }
  }
  game.conds = setbit(COND_HBASE);
}
