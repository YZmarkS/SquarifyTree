#include "render.h"

void drawHorizontalLine(int x1, int x2, int y, int m_x, int m_y, uint32_t* matrix) {
  assert(x1 < x2);
  assert(0 <= x1 && x1 < m_x && 0 <= x2 && x2 < m_x);
  assert(0 <= y && y < m_y);

  long begin = m_x * y + x1;
  long end = m_x * y + x2;

  switch (matrix[begin]) {
  case vl:
    matrix[begin] = lt; break;
  case rt:
    matrix[begin] = cr; break;
  default:
    break;
  }

  for (int x = x1 + 1; x < x2; x++) {
    long position = m_x * y + x;
    switch(matrix[position]) {
    case vl:
      matrix[position] = cr; break;
    case lt:
      matrix[position] = cr; break;
    case rt:
      matrix[position] = cr; break;
    default:
      break;
    }
  }

  switch (matrix[end]) {
  case vl:
    matrix[end] = rt; break;
  case lt:
    matrix[end] = cr; break;
  default:
    break;
  }
}
