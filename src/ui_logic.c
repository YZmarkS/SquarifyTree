#include "ui_logic.h"

CellList* rectangleToCells(Rectangle* rectangle, Coordiate initial) {
  CellList* result = NULL;

  Coordinate curr_top_left = initial;

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

      // TODO insert new node
    }

    // TODO update coordinates for the next split
  }

  // TODO handle the WHOLE rectangle

}

void relateCells(CellList* cells) {
  // TODO computation for relating the cells
}
