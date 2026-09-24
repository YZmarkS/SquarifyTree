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

#define LANDSCAPE_SPLIT_THRESHOLD 4
#define PORTRAIT_SPLIT_THRESHOLD 8

#define COL_MIN_WIDTH 2
#define ROW_MIN_HEIGHT 4

typedef enum StackDirection {
  VERTICAL,
  HORIZONTAL
} StackDirection;

typedef enum RectangleType {
  SPLIT,
  WHOLE
} RectangleType;

typedef enum Orientation {
  LANDSCAPE,
  PORTRAIT
} Orientation;

typedef struct Rectangle Rectangle;

typedef struct Stack Stack;

typedef struct SplitDetail SplitDetail;

typedef struct WholeDetail WholeDetail;

typedef struct Cell Cell;

struct Cell {
  SimpleInode* inode;
  int length;
};

struct Stack {
  int width;
  Cell* cells;
  long num_cells;
};

struct SplitDetail {
  int stack_width;
  long stack_size;
  SimpleInode* inodes;
  int* inode_heights;

  Rectangle* rest;
};

struct WholeDetail {
  SimpleInode* remains;
};

struct Rectangle {
  int width;			/* width */
  int height; 			/* height */

  RectangleType type;
  union {
    SplitDetail split_detail;
    WholeDetail whole_detail;
  } detail;
};

Rectangle* fill(SimpleInode * remains, int width, int height);

void validateRectangle(Rectangle* rectangle, int indent);

#endif
