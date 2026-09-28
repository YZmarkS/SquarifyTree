#ifndef _UI_LOGIC
#define _UI_LOGIC

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

#include "simple_inode.h"
#include "squarify.h"

typedef enum CellType {
  ELEMENT,
  OTHER
} CellType;

typedef struct Coordinate Coordinate;

typedef struct Cell Cell;

typedef struct CellList CellList;

struct Coordinate {
  int x;
  int y;
};

struct Cell {
  Coordinate top_left;
  Coordinate bottom_right;
  struct neightbours {
    Cell* top;
    Cell* bottom;
    Cell* left;
    Cell* right;
  } neighbours;

  SimpleInode* inode;
};

struct CellList {
  Cell* cell_ptr;
  CellList* next;
} CellList;

#endif
