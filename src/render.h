#ifndef _RENDER
#define _RENDER

#ifndef _LARGEFILE64_SOURCE
#define _LARGEFILE64_SOURCE
#endif

#include <stdlib.h>
#include <stdio.h>
#include <stdbool.h>
#include <stdint.h>
#include "simple_inode.h"
#include "algo.h"
#include "squarify.h"

/* typedef struct Position; */

/* struct Position { */
/*   int x; */
/*   int y; */
/* }; */

enum {
    hl = 0x2500, /* ─ */
    vl = 0x2502, /* │ */
    tl = 0x250C, /* ┌ */
    tr = 0x2510, /* ┐ */
    bl = 0x2514, /* └ */
    br = 0x2518, /* ┘ */
    tt = 0x252C, /* ┬ */
    bt = 0x2534, /* ┴ */
    lt = 0x251C, /* ├ */
    rt = 0x2524, /* ┤ */
    cr = 0x253C  /* ┼ */
};

void clearMatrix(int mx, int my, uint32_t* matrix);

void printMatrix(int mx, int my, uint32_t* matrix);

void renderRectangles(Rectangle* rect, int x, int y, int mx, int my, uint32_t* matrix);

void renderLandscape(Rectangle* rect, int x, int y, int mx, int my, uint32_t* matrix);

  void renderPortrait(Rectangle* rect, int x, int y, int mx, int my, uint32_t* matrix);

void drawBoarder(int mx, int my, uint32_t* matrix);

#endif
