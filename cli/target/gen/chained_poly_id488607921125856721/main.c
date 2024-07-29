#include "out.c"
#include <stdio.h>

int main() {
  double x;
  chained_poly_id(&x);
  printf("%lf", x);
}