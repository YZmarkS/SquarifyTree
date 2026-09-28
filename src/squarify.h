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

#define HEIGHT_TO_WIDTH_RATIO 2

#define SPLIT_THRESHOLD 4

#define MIN_RESOLUTION 2

typedef enum RoundingMethod {
  TO_INT,
  TO_EVEN
} RoundingMethod;

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


struct SplitDetail {
  int stack_thickness;
  SimpleInode** inodes;
  int* heights;
  long stack_length;

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

bool isLandScape(int width, int height);

Rectangle* fill(SimpleInode * remains, int width, int height);

void debugRectangle(Rectangle* rectangle, int indent);

#endif
