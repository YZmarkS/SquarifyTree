#include "render.h"

void drawHorizontal(int x, int y, int mx, int my, uint32_t* matrix) {
  long position = mx * y + x;
  switch(matrix[position]) {
    case vl:
      matrix[position] = cr; break;
    case lt:
      matrix[position] = cr; break;
    case rt:
      matrix[position] = cr; break;
    default:
      matrix[position] = hl; break;
  }
}

void drawVertical(int x, int y, int mx, int my, uint32_t* matrix) {
  long position = mx * y + x;
  switch(matrix[position]) {
    case hl:
      matrix[position] = cr; break;
    case tt:
      matrix[position] = cr; break;
    case bt:
      matrix[position] = cr; break;
    default:
      matrix[position] = vl; break;
  }
}

void drawLeft(int x, int y, int mx, int my, uint32_t* matrix) {
  long position = mx * y + x;
  switch(matrix[position]) {
    case vl:
      matrix[position] = rt; break;
    case lt:
      matrix[position] = cr; break;
    default:
      break;
  }
}

void drawRight(int x, int y, int mx, int my, uint32_t* matrix) {
  long position = mx * y + x;
  switch(matrix[position]) {
    case vl:
      matrix[position] = lt; break;
    case rt:
      matrix[position] = cr; break;
    default:
      break;
  }
}

void drawTop(int x, int y, int mx, int my, uint32_t* matrix) {
  long position = mx * y + x;
  switch(matrix[position]) {
    case hl:
      matrix[position] = bt; break;
    case tt:
      matrix[position] = cr; break;
    default:
      break;
  }
}

void drawBottom(int x, int y, int mx, int my, uint32_t* matrix) {
  long position = mx * y + x;
  switch(matrix[position]) {
    case hl:
      matrix[position] = tt; break;
    case bt:
      matrix[position] = cr; break;
    default:
      break;
  }
}

void drawLineHorizontal(int x1, int x2, int y, int mx, int my, uint32_t* matrix) {
  printf("(%d, %d) --- (%d, %d)\n", x1, y, x2, y);
  drawRight(x1, y, mx, my, matrix);
  for (int x = x1 + 1; x < x2; x++) {
    drawHorizontal(x, y, mx, my, matrix);
  }
  drawLeft(x2, y, mx, my, matrix);
}

void drawLineVertical(int x, int y1, int y2, int mx, int my, uint32_t* matrix) {
  printf("(%d, %d) --- (%d, %d)\n", x, y1, x, y2);
  drawBottom(x, y1, mx, my, matrix);
  for (int y = y1 + 1; y < y2; y++) {
    drawVertical(x, y, mx, my, matrix);
  }
  drawTop(x, y2, mx, my, matrix);
}

void drawBoarder(int mx, int my, uint32_t* matrix) {
  for (int x = 1; x < mx - 1; x++) {
    drawHorizontal(x, 0, mx, my, matrix);
    drawHorizontal(x, my - 1, mx, my, matrix);
  }
  for (int y = 1; y < my - 1; y++) {
    drawVertical(0, y, mx, my, matrix);
    drawVertical(mx - 1, y, mx, my, matrix);
  }

  matrix[0] = tl;
  matrix[mx - 1] = tr;
  matrix[(my - 1) * mx] = bl;
  matrix[(my - 1) * mx + mx - 1] = br;

  printf("Done boarder\n");
}

void renderRectangles(Rectangle* rect, int x, int y, int mx, int my, uint32_t* matrix) {
  bool is_landscape = rect->width >= rect->height;
  if (rect->type == WHOLE) {
    return;
  }
  if (rect->type == SPLIT) {
    bool is_landscape = rect->width >= rect->height;
    if (is_landscape) {
      renderLandscape(rect, x, y, mx, my, matrix);
    } else {
      renderPortrait(rect, x, y, mx, my, matrix);
    }
  }
}

void renderLandscape(Rectangle* rect, int x, int y, int mx, int my, uint32_t* matrix) {
  printf("Drawing a Landscape\n");
  SplitDetail split_detail = rect->detail.split_detail;
  int split_line_x = x + split_detail.stack_thickness;
  drawLineVertical(split_line_x, y, my - 1, mx, my, matrix);

  int total_height = 0;
  for (long i = 0; i < split_detail.stack_size - 1; i++) {
    int height = split_detail.heights[i];
    assert(height % 2 == 0 && height >= 4);
    total_height += (height / 2);
    int layer_line_y = y + total_height;
    drawLineHorizontal(x, split_line_x, layer_line_y, mx, my, matrix);
  }
  printf("%d\n", split_detail.heights[split_detail.stack_size - 1] / 2);

  renderRectangles(split_detail.rest, split_line_x, y, mx, my, matrix);
}

void renderPortrait(Rectangle* rect, int x, int y, int mx, int my, uint32_t* matrix) {
  printf("Drawing a Portrait\n");
  SplitDetail split_detail = rect->detail.split_detail;
  int split_line_y = y + (split_detail.stack_thickness / 2);
  drawLineHorizontal(x, mx - 1, split_line_y, mx, my, matrix);

  int total_width = 0;
  for (long i = 0; i < split_detail.stack_size - 1; i++) {
    total_width += split_detail.heights[i];
    int layer_line_x = x + total_width;
    drawLineVertical(layer_line_x, y, split_line_y, mx, my, matrix);
  }

  renderRectangles(split_detail.rest, x, split_line_y, mx, my, matrix);
}

void clearMatrix(int mx, int my, uint32_t* matrix) {
  for (int y = 0; y < my; y++) {
    for (int x = 0; x < mx; x++) {
      matrix[y * mx + x] = 0x20;
    }
  }
}

void printUtf8Char(uint32_t c) {
  if (c < 0x00000080) {
    putchar(c);
  } else if (c < 0x00000800) {
    putchar(0xC0 | (c >> 6));
    putchar(0x80 | (c & 0x3F));
  } else if (c < 0x00010000) {
    putchar(0xE0 | (c >> 12));
    putchar(0x80 | ((c >> 6) & 0x3F));
    putchar(0x80 | (c & 0x3F));
  } else if (c < 0x00200000) {
    putchar(0xF0 | (c >> 18));
    putchar(0x80 | ((c >> 12) & 0x3F));
    putchar(0x80 | ((c >> 6) & 0x3F));
    putchar(0x80 | (c & 0x3F));
  } else if (c < 0x04000000) {
  } else if (c < 0x80000000) {
  }
}

void printMatrix(int mx, int my, uint32_t* matrix) {
  for (int y = 0; y < my; y++) {
    for (int x = 0; x < mx; x++) {
      printUtf8Char(matrix[y * mx + x]);
    }
    putchar('\n');
  }
}
