#include "out.c"
#include <stdio.h>

int main() {
  double x;
  poly_let(&x);
  printf("%lf", x);
}