#include "squarify.h"

int widthBySizeLandscape(off_t size, off_t total_size, int width) {
  double precise_width = ((double) width) * ((double) size) / ((double) total_size);
  int converted_width = round(precise_width);
  return (converted_width >= 1) ? converted_width : 1;
}

int widthBySizePortrait(off_t size, off_t total_size, int width) {
  assert(width % 2 == 0);
  double precise_width = ((double) width) * ((double) size) / ((double) total_size);
  int rounded_width = round(precise_width);
  int converted_width = -1;
  if (rounded_width % 2 == 0) {
    converted_width = rounded_width;
  } else {
    converted_width = (((double)rounded_width) <= precise_width) ? rounded_width + 1 : rounded_width;
  }
  return (converted_width >= 2) ? converted_width : 2;
}

int truncToEven(double f) {
  int n = (int) trunc(f);
  return (n % 2 == 0) ? n : n - 1;
}

int* calcHeightsLandscape(SimpleInode* inodes, long first_n, int total_height) {
  assert(total_height % 2 == 0);
  off_t first_n_size = sumFirstNInodesSize(inodes, first_n);
  int* heights = (int*)calloc(first_n, sizeof(int));
  int remaining_height = total_height;
  SimpleInode* inode = inodes;
  for (long i = 0; i < first_n; i++) {
    double precise_height = (double)total_height * (double)inode->size / (double)first_n_size;
    int height = truncToEven(precise_height);

    heights[i] = height;
    remaining_height -= height;
    inode = inode->next;
  }
  assert(remaining_height % 2 == 0);
  int i = 0;
  while (remaining_height > 0) {
    heights[i] += 2;
    remaining_height -= 2;
    i++;
  }
  assert(remaining_height == 0);
  return heights;
}

int* calcHeightsPortrait(SimpleInode* inodes, long first_n, int total_height) {
  off_t first_n_size = sumFirstNInodesSize(inodes, first_n);
  int* heights = (int*)calloc(first_n, sizeof(int));
  int remaining_height = total_height;
  SimpleInode* inode = inodes;
  for (long i = 0; i < first_n; i++) {
    double precise_height = (double)total_height * ((double)inode->size) / ((double)first_n_size);
    int height = (int)precise_height;

    heights[i] = height;
    remaining_height -= height;
    inode = inode->next;
  }
  int i = 0;
  while (remaining_height > 0) {
    heights[i] += 1;
    remaining_height -= 1;
    i++;
  }
  assert(remaining_height == 0);
  return heights;
}

double worstRatio(int heights[], long length, int width) {
  double width_over_height = ((double) width) / ((double) heights[0]);
  double height_over_width = ((double) heights[0]) / ((double) width);
  double worst = (width_over_height > height_over_width) ? width_over_height : height_over_width;
  for (long i = 1; i < length; i++) {
    width_over_height = ((double) width) / ((double) heights[i]);
    height_over_width = ((double) heights[i]) / ((double) width);
    double ratio = (width_over_height > height_over_width) ? width_over_height : height_over_width;
    worst = (ratio > worst) ? ratio : worst;
  }
  return worst;
}

Rectangle* fill(SimpleInode* remains, int width, int height, Orientation orientation) {
  printf("Width: %d, Height: %d, Orientation: %s\n", width, height, orientation == LANDSCAPE ? "Landscape" : "Portrait");

  int threshold = (orientation == LANDSCAPE) ? LANDSCAPE_SPLIT_THRESHOLD : PORTRAIT_SPLIT_THRESHOLD;
  int right_col_min_width = (orientation == LANDSCAPE) ? LANDSCAPE_RIGHT_COL_MIN_WIDTH : PORTRAIT_RIGHT_COL_MIN_WIDTH;

  Rectangle* new_rectangle = (Rectangle*)calloc(1, sizeof(Rectangle));
  new_rectangle->width = width;
  new_rectangle->height = height;
  new_rectangle->orientation = orientation;

  if (remains == NULL
      || width < threshold) {
    new_rectangle->type = WHOLE;
    new_rectangle->detail.whole_detail.remains = remains;
    return new_rectangle;
  }

  off_t total_size = sumInodeListSize(remains);

  long batch_size = 1;
  off_t curr_size = remains->size;

  int left_col_width = (orientation == LANDSCAPE) ?
    widthBySizeLandscape(curr_size, total_size, width) :
    widthBySizePortrait(curr_size, total_size, width);
  int right_col_width = width - left_col_width;
  int* heights = (orientation == LANDSCAPE) ?
    calcHeightsLandscape(remains, batch_size, height) :
    calcHeightsPortrait(remains, batch_size, height);

  SimpleInode* candidate = remains->next;

  if (right_col_min_width <= right_col_width) { /* Current batch is not going to cause squeeze  */
    while (candidate != NULL) {	                          /* There is a next element */
      printf("%p\n", candidate);
      int new_left_col_width =
	(orientation == LANDSCAPE) ?
	widthBySizeLandscape(curr_size + candidate->size, total_size, width) :
	widthBySizePortrait(curr_size + candidate->size, total_size, width);
      int new_right_col_width = width - new_left_col_width;
      long new_batch_size = batch_size + 1;
      int* new_heights =
	(orientation == LANDSCAPE) ?
	calcHeightsLandscape(remains, new_batch_size, height) :
	calcHeightsPortrait(remains, new_batch_size, height);

      if (new_right_col_width < right_col_min_width /* New batch is not going to cause squeeze */
	  || minIntArray(new_heights, new_batch_size) == 0    /* No cell is too short */
	  || worstRatio(new_heights, new_batch_size, left_col_width) <= worstRatio(heights, batch_size, new_left_col_width) /* ratio does not get worse */
	 ) {
	free(new_heights);
	break;
      }
      batch_size++;
      curr_size += candidate->size;
      candidate = candidate->next;
      free(heights);
      heights = new_heights;
    }
  }

  /* Ensure there is SOMETHING displayed on the right */
  if (right_col_width < right_col_min_width) {
    right_col_width = right_col_min_width;
    left_col_width = width - right_col_width;
  }

  /* Construct left column */
  Column left;
  left.width = left_col_width;
  left.cells = (Cell*)calloc(batch_size, sizeof(Cell));
  left.num_cells = batch_size;
  SimpleInode* curr_inode = remains;
  for (long i = 0; i < batch_size; i++) {
    left.cells[i].inode = curr_inode;
    left.cells[i].height = heights[i];
    curr_inode = curr_inode->next;
  }
  free(heights);

  /* Determine dimension of right section */
  bool need_to_flip = right_col_width < height;

  int new_width = need_to_flip ? height : right_col_width;
  int new_height = need_to_flip ? right_col_width : height;
  Orientation new_orientation =
    need_to_flip ?
    (orientation == LANDSCAPE ? PORTRAIT : LANDSCAPE) :
    orientation;

  printf("Need to flip: %s\n", need_to_flip ? "YES" : "NO");

  Rectangle* right =
      fill(candidate, new_width, new_height, new_orientation);

  SplitDetail split_detail;
  split_detail.left = left;
  split_detail.right = right;

  /* Fill in detail */
  printf("About to split\n");
  new_rectangle->type = SPLIT;
  new_rectangle->detail.split_detail = split_detail;
  return new_rectangle;
}

void padding(int n) {
  for (int i = 0 ; i < n; i++) {
    printf("  ");
  }
}

void validateRectangle(Rectangle* rectangle, int indent) {
  bool is_landscape = rectangle->orientation == LANDSCAPE;
  int x = is_landscape ? rectangle->width : rectangle->height;
  int y = is_landscape ? rectangle->height : rectangle->width;

  padding(indent);
  printf("Expected X: %d    Expected Y: %d    Orientation: %s\n",
	 x,
	 y,
	 rectangle->orientation == LANDSCAPE ? "Landscape" : "Portrait");

  if (rectangle->type == SPLIT) {
    if (is_landscape) {
      // Check column height sum up to y
      int total_y = 0;
      for (long i = 0; i < rectangle->detail.split_detail.left.num_cells; i++) {
	Cell cell = rectangle->detail.split_detail.left.cells[i];
	padding(indent);
	printf("%s height: %d\n", basename(cell.inode->path), cell.height);
	total_y += cell.height;
      }
      padding(indent);
      printf("Column Summed to: %d    Validated: %s\n", total_y, total_y == y ? "True" : "False <==");

      // Check right side height matches
      Rectangle* right = rectangle->detail.split_detail.right;
      bool right_is_landscape = right->orientation == LANDSCAPE;
      int right_x = right_is_landscape ? right->width : right->height;
      int right_y = right_is_landscape ? right->height : right->width;

      padding(indent);
      printf("Right side height: %d    Validated: %s\n", right_y, right_y == y ? "True" : "False <==");

      // Check two sides sum up to x
      padding(indent);
      int total_x = rectangle->detail.split_detail.left.width + right_x;
      printf("Width Summed to: %d    Validated: %s\n", total_x, total_x == x ? "True" : "False <==");

    } else { // is portrait

      int total_x = 0;
      for (long i = 0; i < rectangle->detail.split_detail.left.num_cells; i++) {
	Cell cell = rectangle->detail.split_detail.left.cells[i];
	padding(indent);
	printf("%s width: %d\n", basename(cell.inode->path), cell.height);
	total_x += cell.height;
      }
      padding(indent);
      printf("Row Summed to: %d    Validated: %s\n", total_x, total_x == x ? "True" : "False <==");

      // Check right side height matches
      Rectangle* bottom = rectangle->detail.split_detail.right;
      bool bottom_is_landscape = bottom->orientation == LANDSCAPE;
      int bottom_x = bottom_is_landscape ? bottom->width : bottom->height;
      int bottom_y = bottom_is_landscape ? bottom->height : bottom->width;

      padding(indent);
      printf("Bottom width: %d    Validated: %s\n", bottom_x, bottom_x == x ? "True" : "False <==");

      // Check two sides sum up to x
      padding(indent);
      int total_y = rectangle->detail.split_detail.left.width + bottom_y;
      printf("Height Summed to: %d    Validated: %s\n", total_y, total_y == y ? "True" : "False <==");
    }

    validateRectangle(rectangle->detail.split_detail.right, indent + 1);
  } else if (rectangle->type == WHOLE) {
    SimpleInode* inode = rectangle->detail.whole_detail.remains;
    while (inode != NULL) {
      padding(indent);
      printf("%s\n", basename(inode->path));
      inode = inode->next;
    }
  }
}
