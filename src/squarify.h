#ifndef _SQUARIFY
#define _SQUARIFY

#ifndef _LARGEFILE64_SOURCE
#define _LARGEFILE64_SOURCE
#endif

#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <sys/types.h>
#include <stdbool.h>
#include <unistd.h>
#include <math.h>
#include <limits.h>
#include "simple_inode.h"
#include "algo.h"

#define LANDSCAPE_SPLIT_THRESHOLD 2
#define PORTRAIT_SPLIT_THRESHOLD 4

#define LANDSCAPE_RIGHT_COL_MIN_WIDTH 1
#define PORTRAIT_RIGHT_COL_MIN_WIDTH 2


typedef enum RectangleType {
  SPLIT,
  WHOLE
} RectangleType;

typedef enum Orientation {
  LANDSCAPE,
  PORTRAIT
} Orientation;

typedef struct Rectangle Rectangle;

typedef struct Column Column;

typedef struct SplitDetail;

typedef struct WholeDetail;

typedef struct Cell Cell;

struct Cell {
  SimpleInode* inode;
  int height;
};

struct Column {
  int width;
  Cell* cells;
  long num_cells;
};

struct SplitDetail {
  Column left;
  Rectangle* right;
};

struct WholeDetail {
  SimpleInode* remains;
};

struct Rectangle {
  int width;			/* width */
  int height; 			/* height */
  Orientation orientation;

  RectangleType type;
  union {
    SplitDetail split_detail;
    WholeDetail whole_detail;
  } detail;
};

struct CellList {
  SimpleInode * cell;
  CellList * next;
};


void fill(SimpleInode * remaining, Rectangle* rectangle);

#endif
