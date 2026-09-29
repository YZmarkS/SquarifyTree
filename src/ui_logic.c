#include "ui_logic.h"

CellList* reverseCellList(CellList* list) {
  CellList* front = list;
  CellList* back = NULL;

  while (front != NULL) {
    CellList* front_next = front->next;
    front->next = back;
    back = front;
    front = front_next;
  }

  return back;
}

long cellListLength(CellList* list) {
  long count = 0;
  while (list != NULL) {
    count++;
    list = list->next;
  }
  return count;
}

// Caller responsible for freeing data
CellList* rectangleToCells(Rectangle* rectangle) {
  CellList* head = NULL;

  Coordinate curr_top_left;
  curr_top_left.x = 0;
  curr_top_left.y = 0;

  while (rectangle->type == SPLIT) {
    bool is_landscape = isLandscape(rectangle->width, rectangle->height);
    Coordinate layer_top_left = curr_top_left;
    SplitDetail split_detail = rectangle->detail.split_detail;
    int stack_thickness = split_detail.stack_thickness;

    for (long i = 0; i < split_detail.stack_length; i++) {
      CellList* new_cell_list_node = (CellList*)calloc(1, sizeof(CellList));
      Cell* new_cell = (Cell*)calloc(1, sizeof(Cell));
      new_cell->top_left = layer_top_left;

      Coordinate layer_bottom_right = layer_top_left;
      layer_bottom_right.x += (is_landscape ? stack_thickness : split_detail.heights[i]);
      layer_bottom_right.y += (is_landscape ? split_detail.heights[i] : stack_thickness);

      new_cell->bottom_right = layer_bottom_right;

      new_cell_list_node->cell_ptr = new_cell;
      new_cell_list_node->next = head;
      head = new_cell_list_node;
      layer_top_left.x += (is_landscape ? 0 : split_detail.heights[i]);
      layer_top_left.y += (is_landscape ? split_detail.heights[i] : 0);
    }

    curr_top_left.x += (is_landscape ? stack_thickness : 0);
    curr_top_left.y += (is_landscape ? 0 : stack_thickness);

    rectangle = rectangle->detail.split_detail.rest;
  }

  assert(rectangle->type == WHOLE);
  if (rectangle->detail.whole_detail.remains != NULL) {
    CellList* last_cell_list_node = (CellList*)calloc(1, sizeof(CellList));
    Cell* last_cell = (Cell*)calloc(1, sizeof(Cell));

    Coordinate bottom_right = curr_top_left;
    bottom_right.x += rectangle->width;
    bottom_right.y += rectangle->height;
    last_cell->top_left = curr_top_left;
    last_cell->bottom_right = bottom_right;

    last_cell_list_node->cell_ptr = last_cell;
    last_cell_list_node->next = head;
    head = last_cell_list_node;
  }

  return reverseCellList(head);
}

static int compareCellsOnX(const void* cell_1_ptr, const void* cell_2_ptr) {
  Cell cell1, cell2;
  cell1 = **(Cell**)cell_1_ptr;
  cell2 = **(Cell**)cell_2_ptr;
  if (cell1.bottom_right.x < cell2.bottom_right.x) {
    return -1;
  } else if (cell1.bottom_right.x > cell2.bottom_right.x) {
    return 1;
  }

  if (cell1.top_left.x < cell2.top_left.x) {
    return -1;
  } else if (cell1.top_left.x > cell2.top_left.x) {
    return 1;
  }

  return 0;
}

static int compareCellsOnY(const void* cell_1_ptr, const void* cell_2_ptr) {
  Cell cell1, cell2;
  cell1 = **(Cell**)cell_1_ptr;
  cell2 = **(Cell**)cell_2_ptr;
  if (cell1.bottom_right.y < cell2.bottom_right.y) {
    return -1;
  } else if (cell1.bottom_right.y > cell2.bottom_right.y) {
    return 1;
  }

  if (cell1.top_left.y < cell2.top_left.y) {
    return -1;
  } else if (cell1.top_left.y > cell2.top_left.y) {
    return 1;
  }

  return 0;
}

// Borrow
void relateCells(CellList* cells_list) {
  long length = cellListLength(cells_list);
  Cell** cells_array = (Cell**)calloc(length, sizeof(Cell*));

  CellList* curr_cell_list_node = cells_list;
  for (long i = 0; i < length; i++) {
    cells_array[i] = curr_cell_list_node->cell_ptr;
    curr_cell_list_node = curr_cell_list_node->next;
  }

  qsort(cells_array, length, sizeof(Cell*), compareCellsOnX);

  // TODO fill in left and right neighbour

  qsort(cells_array, length, sizeof(Cell*), compareCellsOnY);

  // TODO fill in top and bottom neighbour

  free(cells_array);
}
