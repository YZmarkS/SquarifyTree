#include "squarify.h"

int widthByRatioLandscape(off_t size, off_t total_size, int width) {
  double precise_width = ((double) width) * ((double) size) / ((double) total_size);
  int converted_width = round(precise_width);
  return (converted_width > 0) ? converted_width : 1;
}

int truncToEven(double f) {
  int n = (int) f;
  return (n % 2 == 0) ? n : n - 1;
}

int* calcHeightsLandscape(SimpleInode* inodes, long first_n, int total_height) {
  assert(total_height % 2 == 0);
  off_t first_n_size = sumFirstNInodeSize(inodes, first_n);
  int* heights = (int*)calloc(first_n, sizeof(int));
  int remaining_height = total_height;
  SimpleInode* inode = inodes;
  for (long i = 0; i < first_n; i++) {
    double precise_height = total_height * ((double)inode->size) / ((double)first_n_size);
    int height = truncToEven(precise_height);

    heights[i] = height;
    remaining_height -= height;
  }
  int i = 0;
  while (remaining_height > 0) {
    heights[i] += 2;
    remaining_height -= 2;
    i++;
  }
  assert(remaining_height == 0);
  return heights;
}

double worstRatio(int heights[], long length, int width) {
  double width_over_height = ((double) width) / ((double) heights[0]);
  double height_over_width = ((double) heights[0]) / ((double) width);
  double worst = (width_over_height > height_over_width) : width_over_height ? height_over_width;
  for (long i = 1; i < length; i++) {
    width_over_height = ((double) width) / ((double) heights[i]);
    height_over_width = ((double) heights[i]) / ((double) width);
    double ratio = (width_over_height > height_over_width) : width_over_height ? height_over_width;
    worst = (ratio > worst) ? ratio : worst;
  }
  return worst;
}

// TODO
/* double worst  */

Rectangle* fillLandscape(SimpleInode* remains, int width, int height) {
  Rectangle* new_rectangle = (Rectangle*)calloc(1, sizeof(Rectangle));
  new_rectangle->width = width;
  new_rectangle->height = height;
  new_rectangle->orientation = LANDSCAPE;

  if (remains == NULL || width < LANDSCAPE_SPLIT_THRESHOLD) {
    new_rectangle->type = WHOLE;
    new_rectangle->detail.whole_detail.remains = remains;
    return new_rectangle;
  }

  off_t total_size = sumInodeListSize(remains);

  long batch_size = 1;
  off_t curr_size = remains->size;

  int left_col_width = widthByRatio(curr_size, total_size, width);
  int right_col_width = width - left_col_width;
  int heights[] = calcHeightsLandscape(remains, batch_size, height);

  SimpleInode* candidate = remains->next;

  /** 1. Check the current batch is not going to cause the right col to be squished


      2. Check there is a next candidate to add
      3. Check adding the candidate is going to to cause the right col to be squished
      5. Check no element in stack is too short
      4. Check aspect ratio is not getting worse

   */
  if (LANDSCAPE_RIGHT_COL_MIN_WIDTH <= right_col_width) { /* Current batch is not going to cause squeeze  */
    while (candidate != NULL) {	                          /* There is a next element */
      int new_left_col_width = widthByRatio(curr_size + candidate->size, total_size, width);
      int new_right_col_width = width - new_left_col_width;
      long new_batch_size = batch_size + 1;
      int new_heights[] = calcHeightsLandscape(remains, new_batch_size, height);

      if (new_right_col_width < LANDSCAPE_RIGHT_COL_MIN_WIDTH /* New batch is not going to cause squeeze */
	  || minIntArray(new_heights, new_batch_size) == 0    /* No cell is too short */
	  || worstRatio(new_heights, new_batch_size, left_col_width) <= worstRatio(heights, batch_size, new_left_col_width) /* ratio does not get worse */
	 ) {
	free(new_heights);
	break;
      }
      batch_size++;
      curr_size = new_size;
      candidate = candidate->next;
      free(heights);
      heights = new_heights;
    }
  }

  if (right_col_width < LANDSCAPE_RIGHT_COL_MIN_WIDTH) {
    right_col_width = 1;
    left_col_width = width - right_col_width;
  }

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

  // TODO calculate remaining rectangle

  free(heights);

}

Rectangle* fillPortrait(SimpleInode* remains, int width, int height) {
  Rectangle* new_rectangle = (Rectangle*)calloc(1, sizeof(Rectangle));
  new_rectangle->width = width;
  new_rectangle->height = height;
  new_rectangle->orientation = PORTRAIT;

  if (remains == NULL || width < PORTRAIT_SPLIT_THRESHOLD) {
    new_rectangle->type = WHOLE;
    new_rectangle->detail.while_detail.remains = remains;
    return new_rectangle;
  }
}
