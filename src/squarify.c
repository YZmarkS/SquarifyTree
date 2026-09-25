#include "squarify.h"

int roundByMethod(double d, RoundingMethod method) {
  int rounded_to_int = round(d);
  int ans = -1;
  if (method == TO_INT) { // Trunc is fine
    ans = rounded_to_int;
  } else if (method == TO_EVEN) {
    int rounded_to_even = -1;
    if (rounded_to_int % 2 == 1) {
      rounded_to_even = rounded_to_int + ((double)rounded_to_int <= d ? 1 : -1);
    } else {
      rounded_to_even = rounded_to_int;
    }
    ans = rounded_to_even;
  }
  return ans;
}

int calcStackThickness(off_t size, off_t total_size, int thickness_to_use, StackDirection direction) {
  bool is_vertical = (direction == VERTICAL) ? true : false;
  bool is_horizontal = !is_vertical;

  // You like the law of excluded middle?
  assert(!is_vertical || thickness_to_use >= LANDSCAPE_SPLIT_THRESHOLD);
  assert(!is_horizontal || thickness_to_use >= PORTRAIT_SPLIT_THRESHOLD);
  assert(!is_horizontal || thickness_to_use % 2 == 0); /* If the stack is vertical */

  double precise_thickness =
    ((double) thickness_to_use) * ((double) size) / ((double) total_size);
  int rounded_thickness = roundByMethod(precise_thickness,
				       is_vertical ? TO_INT : TO_EVEN);

  int corrected_thickness = // prevent stack from being 0 width
    is_vertical ?
    (rounded_thickness >= COL_MIN_WIDTH ? rounded_thickness : COL_MIN_WIDTH) :
    (rounded_thickness >= ROW_MIN_HEIGHT ? rounded_thickness : ROW_MIN_HEIGHT);

  assert(!is_horizontal || corrected_thickness % 2 == 0); /* If the stack is vertical */
  return corrected_thickness;
}

int truncToEven(double f) {
  int n = (int) trunc(f);
  return (n % 2 == 0) ? n : n - 1;
}

int* calcStack(SimpleInode* inodes, long n, int total_height, StackDirection direction) {
  bool is_vertical = (direction == VERTICAL) ? true : false;
  assert(!is_vertical || total_height % 2 == 0);

  off_t total_size = sumFirstNInodesSize(inodes, n);
  int* heights = (int*)calloc(n, sizeof(int));
  int remaining_height = total_height;
  SimpleInode* inode = inodes;
  for (long i = 0; i < n; i++) {
    double precise_height = (double)total_height * (double)inode->size / (double)total_size;
    int height;
    if (is_vertical) {
      height = truncToEven(precise_height);
    } else {
      height = (int)precise_height;
    }
    heights[i] = height;
    remaining_height -= height;
    inode = inode->next;
  }

  assert(!is_vertical || remaining_height % 2 == 0);
  int unit = is_vertical ? 2 : 1;
  long i = 0;
  while (remaining_height > 0) {
    heights[i] += unit;
    remaining_height -= unit;
    i++;
  }
  assert(remaining_height == 0);
  return heights;
}


double worstRatio(int dimension_1[], long length, int dimension_2) {
  double ratio_1 = (double)dimension_2 / (double) dimension_1[0];
  double ratio_2 = (double) dimension_1[0] / (double)dimension_2;
  double ratio = (ratio_1 > ratio_2) ? ratio_1 : ratio_2;
  double worst = ratio;
  for (long i = 1; i < length; i++) {
    double ratio_1 = (double)dimension_2 / (double) dimension_1[0];
    double ratio_2 = (double) dimension_1[0] / (double)dimension_2;
    double ratio = (ratio_1 > ratio_2) ? ratio_1 : ratio_2;
    worst = (ratio > worst) ? ratio : worst;
  }
  return worst;
}

Rectangle* fill(SimpleInode* remains, int width, int height) {
  assert(height % 2 == 0);

  Rectangle* new_rectangle = (Rectangle*)calloc(1, sizeof(Rectangle));
  new_rectangle->width = width;
  new_rectangle->height = height;

  bool is_landscape = width >= height ? true : false;
  /* printf("Width: %d, Height: %d, Orientation: %s\n", width, height, is_landscape ? "Landscape" : "Portrait"); */

  bool has_enough_space =
    is_landscape ?
    (width > LANDSCAPE_SPLIT_THRESHOLD) :
    (height > PORTRAIT_SPLIT_THRESHOLD);

  if (remains == NULL
      || !has_enough_space) {
    new_rectangle->type = WHOLE;
    new_rectangle->detail.whole_detail.remains = remains;
    return new_rectangle;
  }

  off_t total_size = sumInodeListSize(remains);
  long stack_size = 1;
  off_t batch_file_size = remains->size;

  int other_min_thickness = is_landscape ? COL_MIN_WIDTH : ROW_MIN_HEIGHT;
  int stack_thickness =
    is_landscape ?
    calcStackThickness(batch_file_size, total_size, width, VERTICAL) :
    calcStackThickness(batch_file_size, total_size, height, HORIZONTAL);
  int other_thickness =
    is_landscape ?
    (width - stack_thickness) :
    (height - stack_thickness);

  int* heights = calcStack(remains,
			   stack_size,
			   is_landscape ? height : width,
			   is_landscape ? VERTICAL : HORIZONTAL);

  SimpleInode* candidate = remains->next;

  if (other_min_thickness <= other_thickness) {
    while (candidate != NULL) {
      int new_stack_thickness =
	is_landscape ?
	calcStackThickness(batch_file_size + candidate->size, total_size, width, VERTICAL) :
	calcStackThickness(batch_file_size + candidate->size, total_size, height, HORIZONTAL);
      int new_other_thickness =
	is_landscape ?
	(width - new_stack_thickness) :
	(height - new_stack_thickness);
      long new_stack_size = stack_size + 1;
      int* new_heights =
	is_landscape ?
	calcStack(remains, new_stack_size, height, VERTICAL) :
	calcStack(remains, new_stack_size, width, HORIZONTAL);

      bool other_has_space =
	candidate->next == NULL
	|| other_min_thickness <= new_other_thickness;
      bool no_layer_too_short = minIntArray(new_heights, new_stack_size) != 0;
      bool worst_is_worse =
	worstRatio(new_heights, new_stack_size, new_stack_thickness)
	> worstRatio(heights, stack_size, stack_thickness);
      if (!other_has_space || !no_layer_too_short || worst_is_worse) {
	free(new_heights);
	break;
      }

      // Include the candidate into stack
      stack_size = new_stack_size;
      batch_file_size += candidate->size;
      stack_thickness = new_stack_thickness;
      other_thickness = new_other_thickness;
      candidate = candidate->next;
      free(heights);
      heights = new_heights;
    }
  }

  SimpleInode** inodes = (SimpleInode**)calloc(stack_size, sizeof(SimpleInode*));
  SimpleInode* inode = remains;
  for (long i = 0; i < stack_size; i++) {
    inodes[i] = inode;
    inode = inode->next;
  }

  int sub_width = -1, sub_height = -1;
  if (candidate == NULL) {
    assert(stack_thickness == (is_landscape ? width : height));
    sub_width = is_landscape ? 0 : width;
    sub_height = is_landscape ? height : 0;
  } else {
    if (other_thickness < other_min_thickness) {
      other_thickness = other_min_thickness;
      stack_thickness = (is_landscape ? width : height) - other_thickness;
    }
    sub_width = is_landscape ? other_thickness : width;
    sub_height = is_landscape ? height : other_thickness;
  }

  SplitDetail split_detail;
  split_detail.stack_thickness = stack_thickness;
  split_detail.inodes = inodes;
  split_detail.heights = heights;
  split_detail.stack_size = stack_size;
  split_detail.rest = fill(candidate, sub_width, sub_height);

  new_rectangle->type = SPLIT;
  new_rectangle->detail.split_detail = split_detail;
  return new_rectangle;
}

void indenting(int indent) {
  for (int i = 0; i < indent; i++) { printf("  "); }
}

void debugRectangle(Rectangle* rect, int indent) {

  int width = rect->width;
  int height = rect->height;

  bool is_landscape = width >= height;

  indenting(indent);
  printf("Width: %d, Height: %d, Orientation: %s\n", width, height, is_landscape ? "Landscape" : "Portrait");

  if (rect->type == WHOLE) {
    indenting(indent);
    printf("Is Whole\n");
  }

  if (rect->type == SPLIT) {
    Rectangle* rest = rect->detail.split_detail.rest;
    int stack_thickness = rect->detail.split_detail.stack_thickness;
    indenting(indent);
    printf("Stack Thickness: %d    Stack Orientation: %s\n",
	   stack_thickness,
	   is_landscape ? "VERTICAL" : "HORIZONTAL");

    int stack_height = 0;
    for (long i = 0; i < rect->detail.split_detail.stack_size; i++) {
      indenting(indent);
      printf("Layer Name: %s    Layer Height: %d\n",
	     basename(rect->detail.split_detail.inodes[i]->path),
	     rect->detail.split_detail.heights[i]);
      stack_height += rect->detail.split_detail.heights[i];
    }

    indenting(indent);
    printf("Stack Height: %d    Validated: %s\n",
	   stack_height,
	   stack_height == (is_landscape ? height : width) ? "True" : "False <==");

    indenting(indent);
    printf("Other side Height: %d    Validated: %s\n",
	   is_landscape ? rest->height : rest->width,
	   (is_landscape ? rest->height == height : rest->width == width) ? "True" : "False <==");

    int thickness_sum = rect->detail.split_detail.stack_thickness
      + (is_landscape ? rest->width : rest->height);
    int thickness_sum_target = is_landscape ? width : height;
    indenting(indent);
    printf("Thickness Sum: %d    Validated: %s\n",
	   thickness_sum,
	   thickness_sum == thickness_sum_target ? "True" : "False <==");

    debugRectangle(rest, indent + 1);
  }
}
