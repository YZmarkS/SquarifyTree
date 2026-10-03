#include "layout_logic.h"

/* You better make sure the reference to the cells you are
   pointing to is held by someone else, or you get a leak
 */
void freeCellPtrListContainer(CellPtrList* cell_ptr_list) {
  CellPtrList* to_free;
  while (cell_ptr_list != NULL) {
    to_free = cell_ptr_list;
    cell_ptr_list = cell_ptr_list->next;
    free(to_free);
  }
}

void freeCellPtrList(CellPtrList* cell_ptr_list) {
  CellPtrList* to_free;
  while (cell_ptr_list != NULL) {
    to_free = cell_ptr_list;
    cell_ptr_list = cell_ptr_list->next;
    free(to_free->cell_ptr);
    free(to_free);
  }
}

CellPtrList* reverseCellPtrList(CellPtrList* list) {
  CellPtrList* front = list;
  CellPtrList* back = NULL;

  while (front != NULL) {
    CellPtrList* front_next = front->next;
    front->next = back;
    back = front;
    front = front_next;
  }

  return back;
}

long cellListLength(CellPtrList* list) {
  long count = 0;
  while (list != NULL) {
    count++;
    list = list->next;
  }
  return count;
}

// Caller responsible for freeing data
CellPtrList* layoutToCells(const Layout* layout) {
  CellPtrList* head = NULL;

  Coordinate curr_top_left;
  curr_top_left.x = 0;
  curr_top_left.y = 0;

  while (layout->type == SPLIT) {
    bool is_landscape = isLandscape(layout->width, layout->height);
    Coordinate layer_top_left = curr_top_left;
    SplitDetail split_detail = layout->detail.split_detail;
    int stack_thickness = split_detail.stack_thickness;

    for (long i = 0; i < split_detail.stack_length; i++) {
      CellPtrList* new_cell_ptr_list_node = (CellPtrList*)calloc(1, sizeof(CellPtrList));
      Cell* new_cell = (Cell*)calloc(1, sizeof(Cell));

      new_cell->top_left = layer_top_left;

      Coordinate layer_bottom_right = layer_top_left;
      layer_bottom_right.x += (is_landscape ? stack_thickness : split_detail.heights[i]);
      layer_bottom_right.y += (is_landscape ? split_detail.heights[i] : stack_thickness);
      new_cell->bottom_right = layer_bottom_right;

      new_cell->type = ELEMENT;

      new_cell_ptr_list_node->cell_ptr = new_cell;
      new_cell_ptr_list_node->next = head;
      head = new_cell_ptr_list_node;
      layer_top_left.x += (is_landscape ? 0 : split_detail.heights[i]);
      layer_top_left.y += (is_landscape ? split_detail.heights[i] : 0);
    }

    curr_top_left.x += (is_landscape ? stack_thickness : 0);
    curr_top_left.y += (is_landscape ? 0 : stack_thickness);

    layout = layout->detail.split_detail.rest;
  }

  assert(layout->type == WHOLE);
  if (layout->detail.whole_detail.remains != NULL) {
    CellPtrList* last_cell_ptr_list_node = (CellPtrList*)calloc(1, sizeof(CellPtrList));
    Cell* last_cell = (Cell*)calloc(1, sizeof(Cell));

    Coordinate bottom_right = curr_top_left;
    bottom_right.x += layout->width;
    bottom_right.y += layout->height;
    last_cell->top_left = curr_top_left;
    last_cell->bottom_right = bottom_right;

    last_cell->type = OTHER;

    last_cell_ptr_list_node->cell_ptr = last_cell;
    last_cell_ptr_list_node->next = head;
    head = last_cell_ptr_list_node;
  }

  return reverseCellPtrList(head);
}

/* It is up to the caller to ensure the following invariants hold:
   - lower1 <= higher1
   - lower2 <= higher2
*/
int overlapAmount(int lower1, int higher1, int lower2, int higher2) {
  int max_lower = (lower1 >= lower2) ? lower1 : lower2;
  int min_higher = (higher1 <= higher2) ? higher1 : higher2;
  int diff = min_higher - max_lower;
  return diff;
}

int calcOffset(int fixed_lower, int fixed_higher, int lower, int higher) {
  int fixed_mid = (fixed_higher - fixed_lower) / 2 + fixed_lower;
  int mid = (higher - lower) / 2 + lower;
  return abs(fixed_mid - mid);
}

Cell* findMostOverlapping(const CellPtrList* candidates,
			  int lower, int higher,
			  Component component_to_check) {
  bool check_x = component_to_check == ABSCISSA;
  Cell* best = NULL;
  int max_overlap = 0;
  int min_offset_from_mid = INT_MAX;
  while (candidates != NULL) {
    Cell* candidate = candidates->cell_ptr;
    int candidate_lower = (check_x ? candidate->top_left.x : candidate->top_left.y);
    int candidate_higher = (check_x ? candidate->bottom_right.x : candidate->bottom_right.y);

    int overlap = overlapAmount(lower, higher, candidate_lower, candidate_higher);

    if (0 < overlap && max_overlap < overlap) {
      max_overlap = overlap;
      best = candidate;
    } else if (0 < overlap && overlap == max_overlap) {
      int offset_from_mid = calcOffset(lower, higher,
				       candidate_lower, candidate_higher);
      if (offset_from_mid < min_offset_from_mid) {
	best = candidate;
      }
    }
    candidates = candidates->next;
  }
  return best;
}

void groupCellsByIntervalEndpoint(CellPtrList** cells_by_lower,
				  CellPtrList** cells_by_higher,
				  const CellPtrList* cell_ptr_list,
				  Component component) {

  bool group_by_x = component == ABSCISSA;

  const CellPtrList* curr = cell_ptr_list;
  while (curr != NULL) {
    Cell* cell = curr->cell_ptr;
    int lower = group_by_x ? cell->top_left.x : cell->top_left.y;
    int higher = group_by_x ? cell->bottom_right.x : cell->bottom_right.y;

    CellPtrList* lower_cell_ptr_node = (CellPtrList*)calloc(1, sizeof(CellPtrList));
    CellPtrList* higher_cell_ptr_node = (CellPtrList*)calloc(1, sizeof(CellPtrList));
    lower_cell_ptr_node->cell_ptr = cell;
    higher_cell_ptr_node->cell_ptr = cell;

    lower_cell_ptr_node->next = cells_by_lower[lower];
    cells_by_lower[lower] = lower_cell_ptr_node;
    higher_cell_ptr_node->next = cells_by_higher[higher];
    cells_by_higher[higher] = higher_cell_ptr_node;

    curr = curr->next;
  }
}


// Borrow
void relateCells(const CellPtrList* cell_ptr_list,
		 int maximum,
		 RelateDirection relate_direction) {

  bool group_by_x = relate_direction == LEFT_RIGHT;
  bool overlap_by_y = relate_direction == LEFT_RIGHT;
  Component component_to_group_by = relate_direction == LEFT_RIGHT ? ABSCISSA : ORDINATE;
  Component component_to_check = relate_direction == LEFT_RIGHT ? ORDINATE : ABSCISSA;

  CellPtrList** cells_by_lower = (CellPtrList**)calloc(maximum + 1,
						       sizeof(CellPtrList*));
  CellPtrList** cells_by_higher = (CellPtrList**)calloc(maximum + 1,
							sizeof(CellPtrList*));
  for (int i = 0; i < maximum + 1; i++) {
    cells_by_lower[i] = NULL;
    cells_by_higher[i] = NULL;
  }
  groupCellsByIntervalEndpoint(cells_by_lower, cells_by_higher,
			       cell_ptr_list, component_to_group_by);

  const CellPtrList* curr = cell_ptr_list;
  while (curr != NULL) {
    Cell* cell = curr->cell_ptr;
    int lower_group = group_by_x ? cell->top_left.x : cell->top_left.y;
    int higher_group = group_by_x ? cell->bottom_right.x : cell->bottom_right.y;
    int fixed_lower = overlap_by_y ? cell->top_left.y : cell->top_left.x;
    int fixed_higher = overlap_by_y ? cell->bottom_right.y : cell->bottom_right.x;

    Cell* best_neighbour_on_lower_side =
      findMostOverlapping(cells_by_higher[lower_group],
			  fixed_lower, fixed_higher,
			  component_to_check);
    printf("  Found lower side\n");

    Cell* best_neighbour_on_higher_side =
      findMostOverlapping(cells_by_lower[higher_group],
			  fixed_lower, fixed_higher,
			  component_to_check);

    if (relate_direction == LEFT_RIGHT) {
      cell->neighbours.left = best_neighbour_on_lower_side;
      cell->neighbours.right = best_neighbour_on_higher_side;
    } else { // relate_direction == TOP_BOTTOM
      cell->neighbours.top = best_neighbour_on_lower_side;
      cell->neighbours.bottom = best_neighbour_on_higher_side;
    }

    curr = curr->next;

  }

  for (int i = 0; i < maximum + 1; i++) {
    freeCellPtrListContainer(cells_by_lower[i]);
    freeCellPtrListContainer(cells_by_higher[i]);
  }
  free(cells_by_lower);
  free(cells_by_higher);
}
