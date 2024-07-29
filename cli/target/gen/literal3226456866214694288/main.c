#include "out.c"
#include <stdio.h>

int main() {
  double x;
  literal(&x);
  printf("%lf", x);
}