#include "algo.h"

int minIntArray(int* xs, long n) {
  int min = INT_MAX;
  for (long i = 0; i < n; i++) {
    min = (xs[i] < min) ? xs[i] : min;
  }
  return min;
}
