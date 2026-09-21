#ifndef _SQUARIFY
#define _SQUARIFY

#ifndef _LARGEFILE64_SOURCE
#define _LARGEFILE64_SOURCE
#endif

#include <stdlib.h>
#include <stdio.h>
#include <sys/types.h>
#include <stdbool.h>
#include <unistd.h>
#include "walker.h"

typedef enum Orientation {
  LANDSCAPE,
  PORTRAIT
} Orientation;

typedef struct Rectangle Rectangle;
typedef struct Strip Strip;
typedef struct CellList CellList;

struct Rectangle {
  bool is_tail;
  int l; 			/* length */
  int w;			/* width */
  Orientation orientation;

  CellList * cells;
  int stripe_width;

  Rectangle * next;
};

struct CellList {
  SimpleInode * cell;
  CellList * next;
};

typedef struct ArrayifyResult {
  SimpleInode** content;
  long length;
} ArrayifyResult;

void fill(SimpleInode * remaining, Rectangle* rectangle);

#endif
