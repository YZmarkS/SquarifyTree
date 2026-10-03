#include "logic.h"

Cell* move(Cell* current_cell, Direction direction) {
  Cell* next = NULL;
  switch (direction) {
  case L:
    next = current_cell->neighbours.left;
    break;
  case R:
    next = current_cell->neighbours.right;
    break;
  case U:
    next = current_cell->neighbours.top;
    break;
  case D:
    next = current_cell->neighbours.bottom;
    break;
  }
  return (next != NULL ? next : current_cell);
}

ViewModeState enterDir(ViewModeState state) {
  return state;
}

ViewModeState enterOther(ViewModeState state) {
  ViewModeState new_state = state;

  InodePtrList* new_history = (InodePtrList*)calloc(1, sizeof(InodePtrList*));
  new_history->inode = state.starting_inode;
  new_history->next = state.history_stack;
  new_state.history_stack = new_history;

  return new_state;
}


ViewModeState enterCell(ViewModeState state) {
  if (state.active_cell->type == OTHER) {
  } else if (state.active_cell->type == ELEMENT) {
    enterOther(state);
  }
}
