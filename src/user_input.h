#ifndef _USER_INPUT
#define _USER_INPUT

#include <ctype.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <termios.h>
#include <unistd.h>
#include <stddef.h>
#include <sys/types.h>

#include "util.h"

#define CTRL_KEY(k) ((k) & 0x1f)

typedef enum KeyType {
  NOTHING,
  ARROW,
  ENTER,
  BACKSPACE,
  ASCII
} KeyType;

typedef struct KeyPressed KeyPressed;

typedef struct KeyPressed {
  KeyType type;
  union {
    uint8_t ascii_char;
    Direction arrow_direction;
  } key_content;
} KeyPressed;

KeyPressed readOneKeyPress();

#endif
