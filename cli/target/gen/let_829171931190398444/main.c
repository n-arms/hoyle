#include "out.c"
#include <stdio.h>

int main() {
  double x;
  let_(&x);
  printf("%lf", x);
}