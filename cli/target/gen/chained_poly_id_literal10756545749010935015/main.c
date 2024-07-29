#include "out.c"
#include <stdio.h>

int main() {
  double x;
  chained_poly_id_literal(&x);
  printf("%lf", x);
}