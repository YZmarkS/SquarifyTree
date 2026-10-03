#ifndef _LAYOUT
#define _LAYOUT

#ifndef _LARGEFILE64_SOURCE
#define _LARGEFILE64_SOURCE
#endif

#include <stdlib.h>
#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <libgen.h>
#include <string.h>
#include <dirent.h>
#include <stdint.h>

#include "simple_inode.h"
#include "squarify.h"
#include "util.h"

typedef enum Component {
  ABSCISSA,			/* x */
  ORDINATE			/* y */
} Component;

typedef enum RelateDirection {
  LEFT_RIGHT,
  TOP_BOTTOM
} RelateDirection;

typedef enum CellType {
  ELEMENT,
  OTHER
} CellType;

typedef struct Coordinate Coordinate;

typedef struct Cell Cell;

typedef struct CellPtrList CellPtrList;

struct Coordinate {
  int x;
  int y;
};

struct Cell {
  CellType type;
  Coordinate top_left;
  Coordinate bottom_right;

  struct neighbours {
    Cell* top;
    Cell* bottom;
    Cell* left;
    Cell* right;
  } neighbours;

  SimpleInode* inode;
};

struct CellPtrList {
  Cell* cell_ptr;
  CellPtrList* next;
};

CellPtrList* layoutToCells(const Layout* rectangle);

void relateCells(const CellPtrList* cell_ptr_list, int maximum, RelateDirection relate_direction);
#endif
