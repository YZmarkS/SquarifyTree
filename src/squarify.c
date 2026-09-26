#include "squarify.h"

int calcStackThickness(off_t size, off_t total_size, int thickness_to_use) {
  double precise_thickness =
    ((double) thickness_to_use) * ((double) size) / ((double) total_size);
  int rounded_thickness = round(precise_thickness);

  int corrected_thickness =
    rounded_thickness >= MIN_RESOLUTION ?
    rounded_thickness :
    MIN_RESOLUTION;
  return corrected_thickness;
}

int* calcStack(SimpleInode* inodes, long n, int total_height) {
  off_t total_size = sumFirstNInodesSize(inodes, n);
  int* heights = (int*)calloc(n, sizeof(int));
  int remaining_height = total_height;
  SimpleInode* inode = inodes;
  for (long i = 0; i < n; i++) {
    double precise_height = (double)total_height * (double)inode->size / (double)total_size;
    int truncated_height = (int)precise_height;
    heights[i] = truncated_height;
    remaining_height -= truncated_height;
    inode = inode->next;
  }

  long i = 0;
  while (remaining_height > 0) {
    heights[i]++;
    remaining_height--;
    i++;
  }
  assert(remaining_height == 0);
  return heights;
}


double worstRatio(int dimension_1[], long length, int dimension_2, StackDirection direction) {
  const double length_1 = direction == VERTICAL ? PIXEL_ASPECT_RATIO : 1;
  const double length_2 = direction == VERTICAL ? 1 : PIXEL_ASPECT_RATIO;
  double worst = 0;
  for (long i = 0; i < length; i++) {
    // Technically normalize means to the unit distance
    // But we just care about ratio here
    double normalized_1 = ((double)dimension_1[i]) * length_1;
    double normalized_2 = ((double)dimension_2) * length_2;
    double ratio_1 = normalized_1 / normalized_2;
    double ratio_2 = normalized_2 / normalized_1;
    double ratio = (ratio_1 > ratio_2) ? ratio_1 : ratio_2;
    worst = (ratio > worst) ? ratio : worst;
  }
  return worst;
}

Rectangle* fill(SimpleInode* remains, int width, int height) {
  Rectangle* new_rectangle = (Rectangle*)calloc(1, sizeof(Rectangle));
  new_rectangle->width = width;
  new_rectangle->height = height;

  bool is_landscape = width >= (height * PIXEL_ASPECT_RATIO);
  bool enough_to_split = (is_landscape ? width : height) > SPLIT_THRESHOLD;

  if (remains == NULL || !enough_to_split) {
    new_rectangle->type = WHOLE;
    new_rectangle->detail.whole_detail.remains = remains;
    return new_rectangle;
  }

  off_t total_size = sumInodeListSize(remains);
  long stack_length = 1;
  off_t curr_size = remains->size;

  int stack_thickness = calcStackThickness(curr_size,
					   total_size,
					   is_landscape ? width : height);
  int other_thickness = (is_landscape ? width : height) - stack_thickness;

  int* heights = calcStack(remains,
			   stack_length,
			   is_landscape ? height : width);
  double ratio = worstRatio(heights,
			    stack_length,
			    stack_thickness,
			    is_landscape ? VERTICAL : HORIZONTAL);
  printf("%lf\n", ratio);

  SimpleInode* candidate = remains->next;

  if (MIN_RESOLUTION <= other_thickness) {
    while (candidate != NULL) {
      int new_stack_thickness = calcStackThickness(curr_size + candidate->size,
						   total_size,
						   is_landscape ? width : height);
      int new_other_thickness = (is_landscape ? width : height) - new_stack_thickness;
      long new_stack_length = stack_length + 1;
      int* new_heights = calcStack(remains,
				   new_stack_length,
				   is_landscape ? height : width);
      bool other_has_space = candidate->next == NULL || MIN_RESOLUTION <= new_other_thickness;
      bool no_layer_too_short = minIntArray(new_heights, new_stack_length) >= MIN_RESOLUTION;

      double new_ratio = worstRatio(new_heights,
				    new_stack_length,
				    new_stack_thickness,
				    is_landscape ? VERTICAL : HORIZONTAL);
      bool worst_is_worse = ratio < new_ratio;

      printf("Old: %lf, New: %lf\n", ratio, new_ratio);
      if (!other_has_space || !no_layer_too_short || worst_is_worse) {
	free(new_heights);
	break;
      }

      // Include the candidate into stack
      stack_length = new_stack_length;
      curr_size += candidate->size;
      stack_thickness = new_stack_thickness;
      other_thickness = new_other_thickness;
      candidate = candidate->next;
      free(heights);
      heights = new_heights;
      ratio = new_ratio;
    }
  }

  SimpleInode** inodes = (SimpleInode**)calloc(stack_length, sizeof(SimpleInode*));
  SimpleInode* inode = remains;
  for (long i = 0; i < stack_length; i++) {
    inodes[i] = inode;
    inode = inode->next;
  }

  int sub_width = -1, sub_height = -1;
  if (candidate == NULL) {
    assert(stack_thickness == (is_landscape ? width : height));
    sub_width = is_landscape ? 0 : width;
    sub_height = is_landscape ? height : 0;
  } else {
    if (other_thickness < MIN_RESOLUTION) {
      other_thickness = MIN_RESOLUTION;
      stack_thickness = (is_landscape ? width : height) - other_thickness;
    }
    sub_width = is_landscape ? other_thickness : width;
    sub_height = is_landscape ? height : other_thickness;
  }

  SplitDetail split_detail;
  split_detail.stack_thickness = stack_thickness;
  split_detail.inodes = inodes;
  split_detail.heights = heights;
  split_detail.stack_length = stack_length;
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

  bool is_landscape = width >= height * PIXEL_ASPECT_RATIO;

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
    for (long i = 0; i < rect->detail.split_detail.stack_length; i++) {
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
