/*
 * I/O and support routines.
 *
 * SPDX-FileCopyrightText: (C) 1977, 2005 by Will Crowther and Don Woods
 * SPDX-License-Identifier: BSD-2-Clause
 */

#include "advent.h"
#include "bedquilt.h"
#include "dungeon.h"
#include "wasmglk.h"

/*  I/O routines (speak, pspeak, rspeak, sspeak, get_input, yes) */

static char *u32toa(uint32_t x) {
  static char buf[11];
  buf[10] = '\0';
  char *bufptr = buf + 10;

  if (x == 0) {
    bufptr -= 1;
    *bufptr = '0';
    return bufptr;
  }

  while (x > 0) {
    bufptr -= 1;
    *bufptr = (char)(x % 10) + '0';
    x /= 10;
  }

  return bufptr;
}

static void vspeak(const char *msg, bool blank, va_list ap) {
  /* Engine for various speak functions */
  // Do nothing if we got a null pointer or empty string.
  if (msg == NULL || msg[0] == '\0') {
    return;
  }

  if (blank == true) {
    glk_put_char('\n');
  }

  // Handle format specifiers (including the custom %S) by
  // adjusting the parameter accordingly, and replacing the
  // specifier with %s.
  bool pluralize = false;
  for (int i = 0; msg[i] != '\0'; i++) {
    if (msg[i] != '%') {
      /* Ugh.  Least obtrusive way to deal with artifacts "on
       * the floor" being dropped outside of both cave and
       * building. */
      if (strncmp(msg + i, "floor", 5) == 0 && strchr(" .", msg[i + 5]) &&
          !INSIDE(game.loc)) {
        glk_put_string("ground");
        i += 4;
      } else {
        glk_put_char(msg[i]);
      }
    } else {
      i++;
      // Integer specifier.
      if (msg[i] == 'd') {
        uint32_t arg = va_arg(ap, uint32_t);
        glk_put_string(u32toa(arg));
        pluralize = (arg != 1);
      }

      // Unmodified string specifier.
      if (msg[i] == 's') {
        char *arg = va_arg(ap, char *);
        glk_put_string(arg);
      }

      // Singular/plural specifier.
      if (msg[i] == 'S') {
        // look at the *previous* numeric parameter
        if (pluralize) {
          glk_put_char('s');
        }
      }
    }
  }

  glk_put_char('\n');
}

void speak(const char *msg, ...) {
  /* speak a specified string */
  va_list ap;
  va_start(ap, msg);
  vspeak(msg, true, ap);
  va_end(ap);
}

void sspeak(const int msg, ...) {
  /* Speak a message from the arbitrary-messages list */
  va_list ap;
  va_start(ap, msg);
  vspeak(arbitrary_messages[msg], true, ap);
  va_end(ap);
}

void pspeak(vocab_t msg, enum speaktype mode, bool blank, int skip, ...) {
  /* Find the skip+1st message from msg and print it.  Modes are:
   * feel = for inventory, what you can touch
   * look = the full description for the state the object is in
   * listen = the sound for the state the object is in
   * study = text on the object. */
  va_list ap;
  va_start(ap, skip);
  switch (mode) {
  case touch:
    vspeak(objects[msg].inventory, blank, ap);
    break;
  case look:
    vspeak(objects[msg].descriptions[skip], blank, ap);
    break;
  case hear:
    vspeak(objects[msg].sounds[skip], blank, ap);
    break;
  case study:
    vspeak(objects[msg].texts[skip], blank, ap);
    break;
  case change:
    vspeak(objects[msg].changes[skip], blank, ap);
    break;
  }
  va_end(ap);
}

void rspeak(vocab_t i, ...) {
  /* Print the i-th "random" message (section 6 of database). */
  va_list ap;
  va_start(ap, i);
  vspeak(arbitrary_messages[i], true, ap);
  va_end(ap);
}

static char *get_input(void) {
  static char linebuf[LINESIZE + 1];
  event_t ev;

  glk_put_char('\n');

  glk_request_line_event(root_window, 0, LINESIZE, 0);
  do {
    glk_select(&ev);
  } while (ev.type != evtype_LineInput);

  glulx_glkarea_get_bytes(linebuf, 0, ev.val1);
  linebuf[ev.val1] = '\0';
  return linebuf;
}

bool silent_yes_or_no(void) {
  bool outcome = false;

  for (;;) {
    char *reply = get_input();
    if (strlen(reply) == 0) {
      rspeak(PLEASE_ANSWER);
      continue;
    }

    size_t firstword_len = strcspn(reply, " \t");
    reply[firstword_len] = '\0';

    for (size_t i = 0; i < firstword_len; i++) {
      reply[i] = tolower(reply[i]);
    }

    int no = strcmp("no", reply);
    int n = strcmp("n", reply);
    int yes = strcmp("yes", reply);
    int y = strcmp("y", reply);

    if (yes == 0 || y == 0) {
      outcome = true;
      break;
    } else if (no == 0 || n == 0) {
      outcome = false;
      break;
    } else {
      rspeak(PLEASE_ANSWER);
    }
  }
  return outcome;
}

bool yes_or_no(const char *question, const char *yes_response,
               const char *no_response) {
  /*  Print message X, wait for yes/no answer.  If yes, print Y and return
   * true; if no, print Z and return false. */
  bool outcome = false;

  speak(question);

  outcome = silent_yes_or_no();

  if (outcome) {
    speak(yes_response);
  } else {
    speak(no_response);
  }

  return (outcome);
}

/*  Data structure routines */

static int get_motion_vocab_id(const char *word) {
  // Return the first motion number that has 'word' as one of its words.
  for (int i = 0; i < NMOTIONS; ++i) {
    for (int j = 0; j < motions[i].words.n; ++j) {
      if (strncasecmp(word, motions[i].words.strs[j], TOKLEN) == 0 &&
          (strlen(word) > 1 || strchr(ignore, word[0]) == NULL)) {
        return (i);
      }
    }
  }
  // If execution reaches here, we didn't find the word.
  return (WORD_NOT_FOUND);
}

static int get_object_vocab_id(const char *word) {
  // Return the first object number that has 'word' as one of its words.
  for (int i = 0; i < NOBJECTS + 1; ++i) { // FIXME: the + 1 should go when
                                           // 1-indexing for objects is removed
    for (int j = 0; j < objects[i].words.n; ++j) {
      if (strncasecmp(word, objects[i].words.strs[j], TOKLEN) == 0) {
        return (i);
      }
    }
  }
  // If execution reaches here, we didn't find the word.
  return (WORD_NOT_FOUND);
}

static int get_action_vocab_id(const char *word) {
  // Return the first motion number that has 'word' as one of its words.
  for (int i = 0; i < NACTIONS; ++i) {
    for (int j = 0; j < actions[i].words.n; ++j) {
      if (strncasecmp(word, actions[i].words.strs[j], TOKLEN) == 0 &&
          (strlen(word) > 1 || strchr(ignore, word[0]) == NULL)) {
        return (i);
      }
    }
  }
  // If execution reaches here, we didn't find the word.
  return (WORD_NOT_FOUND);
}

static bool is_valid_int(const char *str) {
  /* Returns true if the string passed in is represents a valid integer,
   * that could then be parsed by atoi() */
  // Handle negative number
  if (*str == '-') {
    ++str;
  }

  // Handle empty string or just "-". Should never reach this
  // point, because this is only used with transitive verbs.
  if (!*str) {
    return false; // LCOV_EXCL_LINE
  }

  // Check for non-digit chars in the rest of the string.
  while (*str) {
    if (!isdigit(*str)) {
      return false;
    } else {
      ++str;
    }
  }

  return true;
}

static void get_vocab_metadata(const char *word, vocab_t *id,
                               word_type_t *type) {
  /* Check for an empty string */
  if (strncmp(word, "", sizeof("")) == 0) {
    *id = WORD_EMPTY;
    *type = NO_WORD_TYPE;
    return;
  }

  vocab_t ref_num;

  ref_num = get_motion_vocab_id(word);
  // Second conjunct is because the magic-word placeholder is a bit
  // special
  if (ref_num != WORD_NOT_FOUND) {
    *id = ref_num;
    *type = MOTION;
    return;
  }

  ref_num = get_object_vocab_id(word);
  if (ref_num != WORD_NOT_FOUND) {
    *id = ref_num;
    *type = OBJECT;
    return;
  }

  ref_num = get_action_vocab_id(word);
  if (ref_num != WORD_NOT_FOUND && ref_num != PART) {
    *id = ref_num;
    *type = ACTION;
    return;
  }

  // Check for the reservoir magic word.
  if (strcasecmp(word, game.zzword) == 0) {
    *id = PART;
    *type = ACTION;
    return;
  }

  // Check words that are actually numbers.
  if (is_valid_int(word)) {
    *id = WORD_EMPTY;
    *type = NUMERIC;
    return;
  }

  *id = WORD_NOT_FOUND;
  *type = NO_WORD_TYPE;
  return;
}

bool get_command_input(command_t *command) {
  /* Get user input on stdin, parse and map to command */
  char *input, *first_word, *second_word, *third_word;

  for (;;) {
    input = get_input();

    first_word = strtok(input, "\t ");
    second_word = strtok(NULL, "\t ");
    third_word = strtok(NULL, "\t ");

    if (first_word == NULL) {
      continue;
    }

    if (third_word != NULL) {
      rspeak(TWO_WORDS);
      continue;
    }

    break;
  }

  strncpy(command->word[0].raw, first_word, TOKLEN + TOKLEN);
  strncpy(command->word[1].raw, second_word, TOKLEN + TOKLEN);
  command->word[0].raw[TOKLEN + TOKLEN] = '\0';
  command->word[1].raw[TOKLEN + TOKLEN] = '\0';
  for (int i = 0; command->word[0].raw[i] != '\0'; i++) {
    command->word[0].raw[i] = toupper(command->word[0].raw[i]);
  }
  for (int i = 0; command->word[1].raw[i] != '\0'; i++) {
    command->word[1].raw[i] = toupper(command->word[1].raw[i]);
  }

  get_vocab_metadata(command->word[0].raw, &(command->word[0].id),
                     &(command->word[0].type));
  get_vocab_metadata(command->word[1].raw, &(command->word[1].id),
                     &(command->word[1].type));
  command->state = GIVEN;
  return true;
}

void clear_command(command_t *cmd) {
  /* Resets the state of the command to empty */
  cmd->verb = ACT_NULL;
  cmd->part = unknown;
  game.oldobj = cmd->obj;
  cmd->obj = NO_OBJECT;
  cmd->state = EMPTY;
}

void juggle(obj_t object) {
  /*  Juggle an object by picking it up and putting it down again, the
   * purpose being to get the object to the front of the chain of things
   * at its loc. */
  loc_t i, j;

  i = game.objects[object].place;
  j = game.objects[object].fixed;
  move(object, i);
  move(object + NOBJECTS, j);
}

void move(obj_t object, loc_t where) {
  /*  Place any object anywhere by picking it up and dropping it.  May
   *  already be toting, in which case the carry is a no-op.  Mustn't
   *  pick up objects which are not at any loc, since carry wants to
   *  remove objects from game atloc chains. */
  loc_t from;

  if (object > NOBJECTS) {
    from = game.objects[object - NOBJECTS].fixed;
  } else {
    from = game.objects[object].place;
  }
  /* (ESR) Used to check for !SPECIAL(from). I *think* that was wrong...
   */
  if (from != LOC_NOWHERE && from != CARRIED) {
    carry(object, from);
  }
  drop(object, where);
}

void put(obj_t object, loc_t where, int pval) {
  /*  put() is the same as move(), except it returns a value used to set
   * up the negated game.prop values for the repository objects. */
  move(object, where);
  /* (ESR) Read this in combination with the macro defintions in advent.h.
   */
  game.objects[object].prop = PROP_STASHIFY(pval);
#ifdef OBJECT_SET_SEEN
  OBJECT_SET_SEEN(object);
#endif
}

void carry(obj_t object, loc_t where) {
  /*  Start toting an object, removing it from the list of things at its
   * former location.  Incr holdng unless it was already being toted.  If
   * object>NOBJECTS (moving "fixed" second loc), don't change game.place
   * or game.holdng. */
  int temp;

  if (object <= NOBJECTS) {
    if (game.objects[object].place == CARRIED) {
      return;
    }
    game.objects[object].place = CARRIED;

    /*
     * Without this conditional your inventory is overcounted
     * when you pick up the bird while it's caged. This fixes
     * a cosmetic bug in the original.
     *
     * Possibly this check should be skipped whwn oldstyle is on.
     */
    if (object != BIRD) {
      ++game.holdng;
    }
  }
  if (game.locs[where].atloc == object) {
    game.locs[where].atloc = game.link[object];
    return;
  }
  temp = game.locs[where].atloc;
  while (game.link[temp] != object) {
    temp = game.link[temp];
  }
  game.link[temp] = game.link[object];
}

void drop(obj_t object, loc_t where) {
  /*  Place an object at a given loc, prefixing it onto the game atloc
   * list.  Decr game.holdng if the object was being toted. No state
   * change on the object. */
  if (object > NOBJECTS) {
    game.objects[object - NOBJECTS].fixed = where;
  } else {
    if (game.objects[object].place == CARRIED) {
      if (object != BIRD) {
        /* The bird has to be weightless.  This ugly
         * hack (and the corresponding code in the carry
         * function) brought to you by the fact that
         * when the bird is caged, we need to be able to
         * either 'take bird' or 'take cage' and have
         * the right thing happen.
         */
        --game.holdng;
      }
    }
    game.objects[object].place = where;
  }
  if (where == LOC_NOWHERE || where == CARRIED) {
    return;
  }
  game.link[object] = game.locs[where].atloc;
  game.locs[where].atloc = object;
}

int atdwrf(loc_t where) {
  /*  Return the index of first dwarf at the given location, zero if no
   * dwarf is there (or if dwarves not active yet), -1 if all dwarves are
   * dead.  Ignore the pirate (6th dwarf). */
  int at;

  at = 0;
  if (game.dflag < 2) {
    return at;
  }
  at = -1;
  for (int i = 1; i <= NDWARVES - 1; i++) {
    if (game.dwarves[i].loc == where) {
      return i;
    }
    if (game.dwarves[i].loc != 0) {
      at = 0;
    }
  }
  return at;
}

/*  Utility routines (setbit, tstbit, bug) */

int setbit(int bit) {
  /*  Returns 2**bit for use in constructing bit-masks. */
  return (1L << bit);
}

bool tstbit(int mask, int bit) {
  /*  Returns true if the specified bit is set in the mask. */
  return (mask & (1 << bit)) != 0;
}

// LCOV_EXCL_START
void bug(enum bugtype num, const char *error_string) {
  glk_put_string("Fatal error ");
  glk_put_string(u32toa(num));
  glk_put_string(", ");
  glk_put_string(error_string);
  glk_put_string(".\n");
  glk_exit();
}
// LCOV_EXCL_STOP

void state_change(obj_t obj, int state) {
  /* Object must have a change-message list for this to be useful; only
   * some do */
  game.objects[obj].prop = state;
  pspeak(obj, change, true, state);
}

/* end */
