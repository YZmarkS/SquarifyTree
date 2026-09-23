#include "squarify.h"

int leftColWidthLandscape(SimpleInode* inodes, long first_n, off_t sum, int width) {
  off_t first_n_size = sumFirstNInodeSize(inodes, first_n);
  double ratio = ((double)first_n_size) / ((double)sum);
  double precise_width = ratio * ((double)width);
  int converted_width = round(precise_width);
  return (converted_width > 0) ? converted_width : 1;
}

int rightColWidthLandscape(SimpleInode* inodes, long first_n, off_t sum, int width) {
  return width - rightColWidthLandscape(inodes, first_n, sum, width);
}

int truncToEven(int n) {
  return (n % 2 == 0) ? n : n - 1;
}

int shortestHeightInColLandscape(SimpleInode* inodes, long first_n, int height) {
  off_t first_n_size = sumFirstNInodeSize(inodes, first_n);
  int min_height = INT_MAX;
  SimpleInode* inode = inodes;
  for (int i = 0; i < first_n; i++) {
    double ratio = ((double)inode->size) / ((double)first_n_size);
    double precise_height = ratio * ((double)height);
    int height = truncToEven((int)precise_height);
    min_height = height;
  }
  return height;
}

double worst // TODO

Column calcColumnLandscape // TODO

Rectangle* fillLandscape(SimpleInode* remains, int width, int height) {
  const int
  Rectangle* new_rectangle = (Rectangle*)calloc(1, sizeof(Rectangle));
  new_rectangle->width = width;
  new_rectangle->height = height;
  new_rectangle->orientation = LANDSCAPE;

  if (remains == NULL || width < LANDSCAPE_SPLIT_THRESHOLD) {
    new_rectangle->type = WHOLE;
    new_rectangle->detail.whole_detail.remains = remains;
    return new_rectangle;
  }

  off_t total = sumInodeListSize(remains);
  long batch_size = 1;
  SimpleInode* candidate = remains->next;

  /** 1. Check the current batch is not going to cause the right col to be squished


      2. Check there is a next candidate to add
      3. Check adding the candidate is going to to cause the right col to be squished
      5. Check no element in stack is too short
      4. Check aspect ratio is not getting worse

   */
  if (rightColWidthLandscape(remains, batch_size, total, width) >= LANDSCAPE_RIGHT_COL_MIN_WIDTH) {
    while (candidate != NULL
	   && rightColWidthLandscape(remains, batch_size + 1, total, width) >= LANDSCAPE_RIGHT_COL_MIN_WIDTH
	   && shortestHeightInColLandscape(remains, batch_size + 1, height) > 0
	   // TODO
	   ) {
      batch_size++;
      candidate = candidate->next;
    }
  }

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
